use crate::messages::*;
use anyhow::{Result, anyhow};
use reqwest::Client;
use url::Url;

pub struct SignalingServer {
    url: Url,
    client: Client,
    _keep_local_addresses: bool,
}

impl SignalingServer {
    pub fn new(raw_url: &str, keep_local_addresses: bool) -> Result<Self> {
        let url = Url::parse(raw_url)?;
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        Ok(Self {
            url,
            client,
            _keep_local_addresses: keep_local_addresses,
        })
    }

    pub async fn poll_offer(
        &self,
        sid: &str,
        proxy_type: &str,
        nat_type: &str,
        clients: i32,
        accepted_relay_pattern: Option<String>,
    ) -> Result<ProxyPollResponse> {
        let poll_url = self.url.join("proxy")?;
        let request = ProxyPollRequest {
            sid: sid.to_string(),
            version: VERSION.to_string(),
            proxy_type: proxy_type.to_string(),
            nat: nat_type.to_string(),
            clients,
            accepted_relay_pattern,
        };

        let resp = self.client.post(poll_url)
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Broker returned status code {}", resp.status()));
        }

        let poll_response: ProxyPollResponse = resp.json().await?;
        Ok(poll_response)
    }

    pub async fn send_answer(&self, sid: &str, answer: String) -> Result<bool> {
        let answer_url = self.url.join("answer")?;
        let request = ProxyAnswerRequest {
            version: VERSION.to_string(),
            sid: sid.to_string(),
            answer,
        };

        let resp = self.client.post(answer_url)
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Broker returned status code {}", resp.status()));
        }

        let answer_response: ProxyAnswerResponse = resp.json().await?;
        Ok(answer_response.status == "success")
    }
}
