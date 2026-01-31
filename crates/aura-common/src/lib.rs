//! Shared types for Aura HUD
//!
//! Currently supports Claude Code and Codex. The architecture is designed to support
//! additional agents in future versions.
//!
//! - AgentEvent: Events from code agents
//! - SessionState: Session state machine
//! - IPC protocol
//! - Transcript parsing types and utilities
//!
//! The Claude Code and Codex adapters are in the `adapters` module.

mod event;
mod ipc;
mod session;
pub mod transcript;

pub mod adapters;

pub use event::*;
pub use ipc::*;
pub use session::*;
pub use transcript::*;
