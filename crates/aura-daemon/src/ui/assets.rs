//! Asset source for SVG icons embedded at compile time

use gpui::{AssetSource, Result as GpuiResult, SharedString};
use std::borrow::Cow;

/// Asset source for loading icons from the assets directory
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> GpuiResult<Option<Cow<'static, [u8]>>> {
        // Load from the assets directory embedded at compile time
        let content = match path {
            "icons/terminal.svg" => include_bytes!("../../assets/icons/terminal.svg").as_slice(),
            "icons/book-open.svg" => include_bytes!("../../assets/icons/book-open.svg").as_slice(),
            "icons/pencil.svg" => include_bytes!("../../assets/icons/pencil.svg").as_slice(),
            "icons/file.svg" => include_bytes!("../../assets/icons/file.svg").as_slice(),
            "icons/folder.svg" => include_bytes!("../../assets/icons/folder.svg").as_slice(),
            "icons/search.svg" => include_bytes!("../../assets/icons/search.svg").as_slice(),
            "icons/globe.svg" => include_bytes!("../../assets/icons/globe.svg").as_slice(),
            "icons/plug.svg" => include_bytes!("../../assets/icons/plug.svg").as_slice(),
            "icons/bot.svg" => include_bytes!("../../assets/icons/bot.svg").as_slice(),
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
