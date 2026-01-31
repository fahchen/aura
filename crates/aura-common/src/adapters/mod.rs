//! Adapters for different AI code agents
//!
//! Each adapter converts agent-specific events into generic AgentEvent.
//! Also provides transcript file discovery and parsing for each agent.

pub mod claude_code;
pub mod codex;
