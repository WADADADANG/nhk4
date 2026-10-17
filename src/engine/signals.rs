use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSignal {
    pub id: String,
    #[serde(rename = "actionId")]
    pub action_id: String,
    #[serde(rename = "eventName")]
    pub event_name: String,
    #[serde(rename = "targetId")]
    pub target_id: Option<String>,
    pub timestamp: u64,
}

impl ExecutionSignal {
    pub fn new(action_id: &str, event_name: &str, target_id: Option<&str>) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let id = format!("{}_{}", ts, rand::random::<u16>());

        Self {
            id,
            action_id: action_id.to_string(),
            event_name: event_name.to_string(),
            target_id: target_id.map(|s| s.to_string()),
            timestamp: ts,
        }
    }
}

/// Signal Broadcaster for Realtime UI Energy Pulses & Terminal logs
#[derive(Clone)]
pub struct SignalBroadcaster {
    sender: broadcast::Sender<ExecutionSignal>,
}

impl Default for SignalBroadcaster {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(256);
        Self { sender }
    }
}

impl SignalBroadcaster {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit(&self, action_id: &str, event_name: &str, target_id: Option<&str>) {
        let signal = ExecutionSignal::new(action_id, event_name, target_id);
        let _ = self.sender.send(signal);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ExecutionSignal> {
        self.sender.subscribe()
    }
}
