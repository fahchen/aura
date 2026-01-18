//! Asset source for SVG icons embedded at compile time

use gpui::{AssetSource, Result as GpuiResult, SharedString};
use std::borrow::Cow;

/// Asset source for loading icons from the assets directory
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> GpuiResult<Option<Cow<'static, [u8]>>> {
        // Load from the assets directory embedded at compile time
        let content = match path {
            // State icons (Lucide)
            "icons/cctv.svg" => include_bytes!("../../assets/icons/cctv.svg").as_slice(),
            "icons/message-square-code.svg" => {
                include_bytes!("../../assets/icons/message-square-code.svg").as_slice()
            }
            "icons/bell-ring.svg" => include_bytes!("../../assets/icons/bell-ring.svg").as_slice(),
            "icons/cookie.svg" => include_bytes!("../../assets/icons/cookie.svg").as_slice(),
            "icons/ghost.svg" => include_bytes!("../../assets/icons/ghost.svg").as_slice(),
            "icons/fan.svg" => include_bytes!("../../assets/icons/fan.svg").as_slice(),
            "icons/wind.svg" => include_bytes!("../../assets/icons/wind.svg").as_slice(),

            // Tool icons (Lucide)
            "icons/terminal.svg" => include_bytes!("../../assets/icons/terminal.svg").as_slice(),
            "icons/bot.svg" => include_bytes!("../../assets/icons/bot.svg").as_slice(),
            "icons/book-search.svg" => {
                include_bytes!("../../assets/icons/book-search.svg").as_slice()
            }
            "icons/file-search.svg" => {
                include_bytes!("../../assets/icons/file-search.svg").as_slice()
            }
            "icons/newspaper.svg" => include_bytes!("../../assets/icons/newspaper.svg").as_slice(),
            "icons/file-pen-line.svg" => {
                include_bytes!("../../assets/icons/file-pen-line.svg").as_slice()
            }
            "icons/file-braces.svg" => {
                include_bytes!("../../assets/icons/file-braces.svg").as_slice()
            }
            "icons/monitor-down.svg" => {
                include_bytes!("../../assets/icons/monitor-down.svg").as_slice()
            }
            "icons/binoculars.svg" => {
                include_bytes!("../../assets/icons/binoculars.svg").as_slice()
            }
            "icons/plug.svg" => include_bytes!("../../assets/icons/plug.svg").as_slice(),
            "icons/ticket.svg" => include_bytes!("../../assets/icons/ticket.svg").as_slice(),

            // UI icons (Lucide)
            "icons/audio-lines.svg" => {
                include_bytes!("../../assets/icons/audio-lines.svg").as_slice()
            }
            "icons/bomb.svg" => include_bytes!("../../assets/icons/bomb.svg").as_slice(),
            "icons/x.svg" => include_bytes!("../../assets/icons/x.svg").as_slice(),

            // Indicator icons (Lucide)
            "icons/panda.svg" => include_bytes!("../../assets/icons/panda.svg").as_slice(),
            "icons/wand-sparkles.svg" => {
                include_bytes!("../../assets/icons/wand-sparkles.svg").as_slice()
            }
            "icons/sparkles.svg" => include_bytes!("../../assets/icons/sparkles.svg").as_slice(),
            "icons/flame.svg" => include_bytes!("../../assets/icons/flame.svg").as_slice(),
            "icons/zap.svg" => include_bytes!("../../assets/icons/zap.svg").as_slice(),
            "icons/brain.svg" => include_bytes!("../../assets/icons/brain.svg").as_slice(),
            "icons/spotlight.svg" => include_bytes!("../../assets/icons/spotlight.svg").as_slice(),
            "icons/biceps-flexed.svg" => {
                include_bytes!("../../assets/icons/biceps-flexed.svg").as_slice()
            }
            "icons/rocket.svg" => include_bytes!("../../assets/icons/rocket.svg").as_slice(),
            "icons/cpu.svg" => include_bytes!("../../assets/icons/cpu.svg").as_slice(),
            "icons/puzzle.svg" => include_bytes!("../../assets/icons/puzzle.svg").as_slice(),
            "icons/orbit.svg" => include_bytes!("../../assets/icons/orbit.svg").as_slice(),

            // Legacy icons (kept for compatibility)
            "icons/book-open.svg" => include_bytes!("../../assets/icons/book-open.svg").as_slice(),
            "icons/pencil.svg" => include_bytes!("../../assets/icons/pencil.svg").as_slice(),
            "icons/file.svg" => include_bytes!("../../assets/icons/file.svg").as_slice(),
            "icons/folder.svg" => include_bytes!("../../assets/icons/folder.svg").as_slice(),
            "icons/search.svg" => include_bytes!("../../assets/icons/search.svg").as_slice(),
            "icons/globe.svg" => include_bytes!("../../assets/icons/globe.svg").as_slice(),
            "icons/settings.svg" => include_bytes!("../../assets/icons/settings.svg").as_slice(),
            "icons/check.svg" => include_bytes!("../../assets/icons/check.svg").as_slice(),
            "icons/bell.svg" => include_bytes!("../../assets/icons/bell.svg").as_slice(),
            _ => return Ok(None),
        };
        Ok(Some(Cow::Borrowed(content)))
    }

    fn list(&self, _path: &str) -> GpuiResult<Vec<SharedString>> {
        Ok(vec![])
    }
}
