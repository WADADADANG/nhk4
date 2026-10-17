use image::{load_from_memory, DynamicImage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Frame cache for screencasts per client (1-8)
#[derive(Clone, Default)]
pub struct ScreencastCache {
    frames: Arc<RwLock<HashMap<u32, (DynamicImage, u64)>>>,
}

impl ScreencastCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Decode JPEG frame buffer into cached DynamicImage
    pub async fn update_frame(&self, client_id: u32, jpeg_data: &[u8]) -> Result<(), String> {
        let img = load_from_memory(jpeg_data).map_err(|e| e.to_string())?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut lock = self.frames.write().await;
        lock.insert(client_id, (img, now));
        Ok(())
    }

    pub async fn get_frame(&self, client_id: u32) -> Option<DynamicImage> {
        let lock = self.frames.read().await;
        lock.get(&client_id).map(|(img, _)| img.clone())
    }
}
