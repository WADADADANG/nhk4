use rdev::{Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum HotkeyAction {
    ToggleSuspend,
    TriggerKey(String),
}

/// Global Hotkey Listener (rdev Cross-Platform implementation)
pub struct GlobalHotkeyListener {
    is_suspended: Arc<AtomicBool>,
}

impl GlobalHotkeyListener {
    pub fn new() -> Self {
        Self {
            is_suspended: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_suspended(&self) -> bool {
        self.is_suspended.load(Ordering::SeqCst)
    }

    pub fn set_suspended(&self, val: bool) {
        self.is_suspended.store(val, Ordering::SeqCst);
    }

    /// Start listening in a background OS thread
    pub fn start_listener(
        &self,
        suspend_key: String,
        action_sender: broadcast::Sender<HotkeyAction>,
    ) -> Result<(), String> {
        let suspended = self.is_suspended.clone();
        let target_suspend = suspend_key.to_uppercase();

        std::thread::spawn(move || {
            let callback = move |event: Event| {
                if let EventType::KeyPress(key) = event.event_type {
                    let key_str = format!("{:?}", key);
                    let is_match = match key {
                        Key::End if target_suspend == "END" => true,
                        Key::F1 if target_suspend == "F1" => true,
                        Key::F2 if target_suspend == "F2" => true,
                        Key::F12 if target_suspend == "F12" => true,
                        _ => key_str.to_uppercase() == target_suspend,
                    };

                    if is_match {
                        let prev = suspended.fetch_xor(true, Ordering::SeqCst);
                        let _ = action_sender.send(HotkeyAction::ToggleSuspend);
                        println!(
                            "🛑 [Global Hotkey] Suspend toggled: now {}",
                            if !prev { "PAUSED" } else { "RUNNING" }
                        );
                    } else {
                        let _ = action_sender.send(HotkeyAction::TriggerKey(key_str));
                    }
                }
            };

            if let Err(error) = rdev::listen(callback) {
                eprintln!("⚠️ [Global Hotkey] Error listening to input: {:?}", error);
            }
        });

        Ok(())
    }
}
