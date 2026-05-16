mod messages;
mod broker;
mod util;
mod tokens;
mod event;
mod relay;
mod nat;

use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use anyhow::{Result, Context};
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::api::APIBuilder;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::broker::SignalingServer;
use crate::tokens::Tokens;
use crate::event::SnowflakeEventDispatcher;
use crate::nat::{NATType};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "https://snowflake-broker.torproject.net/")]
    broker: String,

    #[arg(short, long, default_value_t = 0)]
    capacity: u32,

    #[arg(short, long, default_value = "wss://snowflake.bamsoftware.com/")]
    relay: String,

    #[arg(short, long, default_value = "stun:stun.stunprotocol.org:3478")]
    stun: String,

    #[arg(long)]
    keep_local_addresses: bool,
}

async fn run_session(
    sid: String,
    broker: Arc<SignalingServer>,
    tokens: Arc<Tokens>,
    event_dispatcher: Arc<SnowflakeEventDispatcher>,
    relay_url_default: String,
    stun_url: String,
    nat_type_mgr: Arc<NATType>,
) -> Result<()> {
    let _permit = tokens.get().await;

    let nat_type = nat_type_mgr.get().await;
    let poll_resp = broker.poll_offer(&sid, "standalone", &nat_type, tokens.count() as i32, None).await?;

    if poll_resp.status == "no match" {
        tokens.ret();
        return Ok(());
    }

    let offer = poll_resp.offer.context("No offer in response")?;
    let relay_url = poll_resp.relay_url.unwrap_or(relay_url_default);

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec![stun_url],
            ..Default::default()
        }],
        ..Default::default()
    };

    let s = SettingEngine::default();
    let api = APIBuilder::new().with_setting_engine(s).build();
    let pc = Arc::new(api.new_peer_connection(config).await?);

    let (dc_tx, mut dc_rx) = tokio::sync::mpsc::channel(1);

    pc.on_data_channel(Box::new(move |dc| {
        let dc_tx = dc_tx.clone();
        Box::pin(async move {
            let _ = dc_tx.send(dc).await;
        })
    }));

    let offer_desc = RTCSessionDescription::offer(offer)?;
    pc.set_remote_description(offer_desc).await?;

    let answer = pc.create_answer(None).await?;
    let mut gather_complete = pc.gathering_complete_promise().await;
    pc.set_local_description(answer).await?;
    let _ = gather_complete.recv().await;

    if let Some(local_desc) = pc.local_description().await {
        let mut sdp = local_desc.sdp;
        if !broker.keep_local_addresses {
            sdp = util::strip_local_addresses(&sdp);
        }
        broker.send_answer(&sid, sdp).await?;
    }

    let pc_clone = pc.clone();
    let relay_url_clone = relay_url.clone();
    let event_dispatcher_clone = event_dispatcher.clone();
    let tokens_clone = tokens.clone();

    tokio::spawn(async move {
        if let Some(dc) = dc_rx.recv().await {
            let _ = relay::copy_loop(dc, relay_url_clone, event_dispatcher_clone).await;
        }
        let _ = pc_clone.close().await;
        tokens_clone.ret();
    });

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    let broker = Arc::new(SignalingServer::new(&args.broker, args.keep_local_addresses)?);
    let tokens = Arc::new(Tokens::new(args.capacity));
    let event_dispatcher = Arc::new(SnowflakeEventDispatcher::new());
    let nat_type_mgr = Arc::new(NATType::new());

    println!("Snowflake Proxy in Rust started");

    let mut ticker = time::interval(Duration::from_secs(5));
    loop {
        ticker.tick().await;
        let sid = util::gen_session_id();
        let broker = broker.clone();
        let tokens = tokens.clone();
        let event_dispatcher = event_dispatcher.clone();
        let relay_url = args.relay.clone();
        let stun_url = args.stun.clone();
        let nat_type_mgr = nat_type_mgr.clone();

        tokio::spawn(async move {
            if let Err(e) = run_session(sid, broker, tokens, event_dispatcher, relay_url, stun_url, nat_type_mgr).await {
                log::error!("Session error: {}", e);
            }
        });
    }
}
