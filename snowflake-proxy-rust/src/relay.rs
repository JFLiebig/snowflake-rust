use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures::{StreamExt, SinkExt};
use anyhow::{Result, Context};
use webrtc::data_channel::RTCDataChannel;
use crate::event::SnowflakeEventDispatcher;
use tokio::sync::mpsc;

pub async fn copy_loop(
    dc: Arc<RTCDataChannel>,
    relay_url: String,
    _dispatcher: Arc<SnowflakeEventDispatcher>,
) -> Result<()> {
    let (ws_stream, _) = connect_async(relay_url).await.context("Failed to connect to relay")?;
    let (mut ws_sink, mut ws_source) = ws_stream.split();

    let (dc_tx, mut dc_rx) = mpsc::channel(32);

    dc.on_message(Box::new(move |msg| {
        let dc_tx = dc_tx.clone();
        Box::pin(async move {
            let _ = dc_tx.send(msg.data.to_vec()).await;
        })
    }));

    let dc_to_ws = async {
        while let Some(data) = dc_rx.recv().await {
            ws_sink.send(Message::Binary(data)).await?;
        }
        Ok::<(), anyhow::Error>(())
    };

    let ws_to_dc = async {
        while let Some(msg) = ws_source.next().await {
            let data = msg?.into_data();
            dc.send(&data.into()).await?;
        }
        Ok::<(), anyhow::Error>(())
    };

    tokio::select! {
        res = dc_to_ws => res,
        res = ws_to_dc => res,
    }
}
