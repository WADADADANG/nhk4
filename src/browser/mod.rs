pub mod anti_stuck;
pub mod cdp;
pub mod cdp_session;
pub mod client_manager;
pub mod jitter;
pub mod launcher;
pub mod window_bounds;

pub use anti_stuck::ANTI_STUCK_INJECTION_SCRIPT;
pub use cdp::CdpRequest;
pub use cdp_session::CdpSession;
pub use client_manager::{ClientManager, ClientSlot};
pub use jitter::GhostMouseJitter;
pub use launcher::{BrowserLauncher, BrowserType};
pub use window_bounds::{get_client_window_bounds_from_pid, save_client_bounds_if_changed};
