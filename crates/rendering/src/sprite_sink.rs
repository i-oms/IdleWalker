//! The presenter's output port: where a finished frame is shown.
use crate::surface::Surface;

/// One borderless, click-through, always-on-top window showing one sprite. The
/// platform crate implements it for each OS; whoever owns the frame loop is handed a
/// factory for them, so nothing here or in the UI names an OS.
pub trait SpriteSink {
    /// Shows the surface with its top-left corner at (x, y), in world pixels.
    fn present(&mut self, surface: Surface, x: i32, y: i32);
    /// Raises the window above anything that covered it.
    fn stay_on_top(&self);
    fn hide(&mut self);
}

/// Makes sprite windows on demand.
pub type SpriteFactory = std::sync::Arc<dyn Fn() -> Box<dyn SpriteSink> + Send + Sync>;
