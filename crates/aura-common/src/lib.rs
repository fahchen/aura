//! Shared types for Aura HUD
//!
//! Currently supports Claude Code. The architecture is designed to support
//! additional agents in future versions.
//!
//! - AgentEvent: Events from code agents
//! - SessionState: Session state machine
//! - IPC protocol
//!
//! The Claude Code adapter is in the `adapters` module.

mod event;
mod ipc;
mod session;

pub mod adapters;

pub use event::*;
pub use ipc::*;
pub use session::*;
