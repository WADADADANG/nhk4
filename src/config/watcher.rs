use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum ConfigChangeEvent {
    ProfileModified(String),
    GlobalModified,
}

/// Watcher using kernel filesystem events (inotify on Linux, ReadDirectoryChangesW on Windows, FSEvents on macOS)
pub struct ConfigWatcher {
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    pub fn start<P: AsRef<Path>>(
        watch_dir: P,
        event_sender: broadcast::Sender<ConfigChangeEvent>,
    ) -> Result<Self, String> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )
        .map_err(|e| e.to_string())?;

        watcher
            .watch(watch_dir.as_ref(), RecursiveMode::Recursive)
            .map_err(|e| e.to_string())?;

        // Background handler thread
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                for path in event.paths {
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        if file_name == "global.json" {
                            let _ = event_sender.send(ConfigChangeEvent::GlobalModified);
                        } else if file_name.ends_with(".json") {
                            let profile_name = file_name.trim_end_matches(".json").to_string();
                            let _ = event_sender.send(ConfigChangeEvent::ProfileModified(profile_name));
                        }
                    }
                }
            }
        });

        Ok(Self { _watcher: watcher })
    }
}
