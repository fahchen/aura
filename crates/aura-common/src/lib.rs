//! Shared types for Aura HUD
//!
//! This crate contains agent-agnostic types:
//! - AgentEvent: Generic events from any code agent
//! - SessionState: Session state machine
//! - IPC protocol
//!
//! Agent-specific adapters are in the `adapters` module.

mod event;
mod ipc;
mod session;

pub mod adapters;

pub use event::*;
pub use ipc::*;
pub use session::*;
