use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSlot {
    pub id: u32,
    pub is_running: bool,
    pub is_paused: bool,
    pub alias: Option<String>,
    pub proxy: Option<String>,
    pub user_agent: Option<String>,
    pub debug_port: Option<u16>,
    pub pid: Option<u32>,
}

#[derive(Clone, Default)]
pub struct ClientManager {
    slots: Arc<RwLock<HashMap<u32, ClientSlot>>>,
}

impl ClientManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_all_clients(&self) -> Vec<ClientSlot> {
        let lock = self.slots.read().await;
        let mut list: Vec<ClientSlot> = lock.values().cloned().collect();
        list.sort_by_key(|c| c.id);
        list
    }

    pub async fn get_client(&self, id: u32) -> Option<ClientSlot> {
        let lock = self.slots.read().await;
        lock.get(&id).cloned()
    }

    pub async fn register_client(&self, slot: ClientSlot) {
        let mut lock = self.slots.write().await;
        lock.insert(slot.id, slot);
    }

    pub async fn set_running(&self, id: u32, is_running: bool, debug_port: Option<u16>, pid: Option<u32>) {
        let mut lock = self.slots.write().await;
        if let Some(client) = lock.get_mut(&id) {
            client.is_running = is_running;
            if debug_port.is_some() {
                client.debug_port = debug_port;
            }
            if pid.is_some() {
                client.pid = pid;
            }
        } else {
            lock.insert(
                id,
                ClientSlot {
                    id,
                    is_running,
                    is_paused: false,
                    alias: None,
                    proxy: None,
                    user_agent: None,
                    debug_port,
                    pid,
                },
            );
        }
    }

    pub async fn set_paused(&self, id: u32, is_paused: bool) {
        let mut lock = self.slots.write().await;
        if let Some(client) = lock.get_mut(&id) {
            client.is_paused = is_paused;
        } else {
            lock.insert(
                id,
                ClientSlot {
                    id,
                    is_running: false,
                    is_paused,
                    alias: None,
                    proxy: None,
                    user_agent: None,
                    debug_port: None,
                    pid: None,
                },
            );
        }
    }

    pub async fn is_running(&self, id: u32) -> bool {
        let lock = self.slots.read().await;
        lock.get(&id).map(|c| c.is_running).unwrap_or(false)
    }
}
