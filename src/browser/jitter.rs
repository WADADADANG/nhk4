use crate::browser::cdp_session::CdpSession;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Background worker for Ghost Mouse Jitter (Anti-AFK Protection)
pub struct GhostMouseJitter;

impl GhostMouseJitter {
    pub fn start_worker(
        session: CdpSession,
        interval_min_ms: u64,
        interval_max_ms: u64,
        max_offset: i32,
        is_active: Arc<AtomicBool>,
    ) {
        tokio::spawn(async move {
            let mut current_x = 400.0f64;
            let mut current_y = 300.0f64;

            while is_active.load(Ordering::SeqCst) {
                let delay = {
                    let mut rng = rand::thread_rng();
                    if interval_max_ms > interval_min_ms {
                        rng.gen_range(interval_min_ms..=interval_max_ms)
                    } else {
                        interval_min_ms.max(5000)
                    }
                };

                tokio::time::sleep(Duration::from_millis(delay)).await;

                if !is_active.load(Ordering::SeqCst) {
                    break;
                }

                // Random slight delta
                let (dx, dy) = {
                    let mut rng = rand::thread_rng();
                    (
                        rng.gen_range(-max_offset..=max_offset) as f64,
                        rng.gen_range(-max_offset..=max_offset) as f64,
                    )
                };

                current_x = (current_x + dx).clamp(100.0, 700.0);
                current_y = (current_y + dy).clamp(100.0, 500.0);

                let _ = session.send_mouse_move(current_x, current_y).await;
            }
        });
    }
}
