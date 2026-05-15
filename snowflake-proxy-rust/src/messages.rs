use serde::{Deserialize, Serialize};

pub const VERSION: &str = "1.3";
pub const PROXY_UNKNOWN: &str = "unknown";

#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyPollRequest {
    #[serde(rename = "Sid")]
    pub sid: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Type")]
    pub proxy_type: String,
    #[serde(rename = "NAT")]
    pub nat: String,
    #[serde(rename = "Clients")]
    pub clients: i32,
    #[serde(rename = "AcceptedRelayPattern")]
    pub accepted_relay_pattern: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyPollResponse {
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Offer")]
    #[serde(default)]
    pub offer: Option<String>,
    #[serde(rename = "NAT")]
    #[serde(default)]
    pub nat: Option<String>,
    #[serde(rename = "RelayURL")]
    #[serde(default)]
    pub relay_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyAnswerRequest {
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Sid")]
    pub sid: String,
    #[serde(rename = "Answer")]
    pub answer: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyAnswerResponse {
    #[serde(rename = "Status")]
    pub status: String,
}
