use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannel {
    Sfx,
    Tts,
}

enum AudioCommand {
    Play {
        channel: AudioChannel,
        path: PathBuf,
        volume: f32,
        interrupt: bool,
    },
    Stop(Option<AudioChannel>),
}

/// Native Cross-Platform Audio Player with Dedicated Audio Thread
/// Supports dual-channel playback (SFX & TTS) with sequential queueing & instant interrupt
#[derive(Clone)]
pub struct AudioPlayer {
    tx: Sender<AudioCommand>,
}

impl AudioPlayer {
    pub fn try_new() -> Result<Self, String> {
        let (tx, rx) = channel::<AudioCommand>();

        std::thread::spawn(move || {
            let stream_res = OutputStream::try_default();
            let (_stream, stream_handle) = match stream_res {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("⚠️ [Audio Thread] Failed to open default audio output: {}", e);
                    return;
                }
            };

            let mut tts_sink: Option<Sink> = None;
            let mut sfx_sink: Option<Sink> = None;

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    AudioCommand::Play { channel, path, volume, interrupt } => {
                        let vol = volume.clamp(0.0, 1.0);
                        let target_sink = match channel {
                            AudioChannel::Tts => &mut tts_sink,
                            AudioChannel::Sfx => &mut sfx_sink,
                        };

                        if interrupt {
                            // Cut Mode: Immediately stop previous audio on this channel
                            if let Some(sink) = target_sink.take() {
                                sink.stop();
                            }
                        }

                        if let Ok(file) = File::open(&path) {
                            let reader = BufReader::new(file);
                            if let Ok(source) = Decoder::new(reader) {
                                if !interrupt {
                                    // Queue Mode (waitForPrevious): append to active sink if currently playing
                                    let is_active = target_sink.as_ref().map(|s| !s.empty()).unwrap_or(false);
                                    if is_active {
                                        if let Some(sink) = target_sink {
                                            sink.append(source.amplify(vol));
                                            continue;
                                        }
                                    }
                                }

                                // Create fresh sink if cut mode or no sound currently in queue
                                if let Ok(sink) = Sink::try_new(&stream_handle) {
                                    sink.set_volume(1.0);
                                    sink.append(source.amplify(vol));
                                    *target_sink = Some(sink);
                                }
                            }
                        }
                    }
                    AudioCommand::Stop(channel_opt) => {
                        match channel_opt {
                            Some(AudioChannel::Tts) => {
                                if let Some(sink) = tts_sink.take() {
                                    sink.stop();
                                }
                            }
                            Some(AudioChannel::Sfx) => {
                                if let Some(sink) = sfx_sink.take() {
                                    sink.stop();
                                }
                            }
                            None => {
                                if let Some(sink) = tts_sink.take() {
                                    sink.stop();
                                }
                                if let Some(sink) = sfx_sink.take() {
                                    sink.stop();
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Self { tx })
    }

    /// Play TTS voice with sequential queueing or instant interrupt
    pub fn play_tts<P: AsRef<Path>>(&self, path: P, volume: f32, wait_for_previous: bool) {
        let _ = self.tx.send(AudioCommand::Play {
            channel: AudioChannel::Tts,
            path: path.as_ref().to_path_buf(),
            volume,
            interrupt: !wait_for_previous,
        });
    }

    /// Play sound effects (SFX) without cutting or blocking TTS
    pub fn play_sfx<P: AsRef<Path>>(&self, path: P, volume: f32, interrupt: bool) {
        let _ = self.tx.send(AudioCommand::Play {
            channel: AudioChannel::Sfx,
            path: path.as_ref().to_path_buf(),
            volume,
            interrupt,
        });
    }

    /// Legacy / general sound play method
    pub fn play_sound<P: AsRef<Path>>(&self, path: P, volume: f32, interrupt: bool) {
        self.play_sfx(path, volume, interrupt);
    }

    /// Stop all audio channels
    pub fn stop(&self) {
        let _ = self.tx.send(AudioCommand::Stop(None));
    }

    /// Stop only TTS audio channel
    pub fn stop_tts(&self) {
        let _ = self.tx.send(AudioCommand::Stop(Some(AudioChannel::Tts)));
    }

    /// Stop only SFX audio channel
    pub fn stop_sfx(&self) {
        let _ = self.tx.send(AudioCommand::Stop(Some(AudioChannel::Sfx)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_player_commands() {
        if let Ok(player) = AudioPlayer::try_new() {
            player.play_tts("test.mp3", 0.8, true);
            player.play_tts("test2.mp3", 0.8, false);
            player.play_sfx("ding.wav", 1.0, true);
            player.stop_tts();
            player.stop_sfx();
            player.stop();
        }
    }
}
