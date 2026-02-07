//! Aura — HUD for AI coding agents

mod event;
pub mod ipc;
mod session;

pub use event::*;
pub use session::*;

pub mod agents;
pub mod registry;
pub mod server;
pub mod ui;
