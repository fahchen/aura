//! HUD UI module - gpui-based notch-flanking icons
//!
//! Two small windows positioned on either side of the macOS notch:
//! - Left: Attention bell (yellow) or Check (green)
//! - Right: State icon (Running/Compacting/Idle/Stale)

mod icons;
mod state;
mod window;

use crate::registry::SessionRegistry;
use std::sync::{Arc, Mutex};

pub fn run_hud(registry: Arc<Mutex<SessionRegistry>>) {
    window::run_hud(registry);
}
