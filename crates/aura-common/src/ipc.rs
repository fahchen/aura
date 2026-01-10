//! IPC message protocol between adapters and daemon

use crate::{AgentEvent, SessionState};
use serde::{Deserialize, Serialize};

/// Socket name for IPC communication
pub const SOCKET_NAME: &str = "aura.sock";

/// Message from adapter to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcMessage {
    /// Agent event (generic, from any adapter)
    Event(AgentEvent),
    /// Ping to check if daemon is alive
    Ping,
}

/// Response from daemon to adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcResponse {
    /// Acknowledgment
    Ok,
    /// Pong response to ping
    Pong,
    /// Error message
    Error { message: String },
}

/// Session information for IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub cwd: String,
    pub state: SessionState,
    pub running_tools: Vec<RunningTool>,
}

/// A currently running tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningTool {
    pub tool_id: String,
    pub tool_name: String,
}
