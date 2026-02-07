//! Shared types for Aura HUD
//!
//! Currently supports Claude Code and Codex.
//!
//! - AgentEvent: Events from code agents
//! - SessionState: Session state machine
//! - IPC: Message types for hook → daemon communication

mod event;
pub mod ipc;
mod session;

pub use event::*;
pub use session::*;
