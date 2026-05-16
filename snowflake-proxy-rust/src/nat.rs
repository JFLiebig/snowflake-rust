use tokio::sync::RwLock;
use crate::broker::NAT_UNKNOWN;

pub struct NATType {
    current: RwLock<String>,
}

impl NATType {
    pub fn new() -> Self {
        Self {
            current: RwLock::new(NAT_UNKNOWN.to_string()),
        }
    }

    pub async fn get(&self) -> String {
        self.current.read().await.clone()
    }
}
