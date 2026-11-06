//! The presenter: turns walkers into pixels. It knows the domain and nothing outside
//! it; finished frames leave through the `SpriteSink` port, which the platform crate
//! implements with a real window on each OS.
#![allow(warnings)]
pub mod bird_painter;
pub mod frame_renderer;
pub mod minion_painter;
pub mod render_thread;
pub mod solid_painter;
pub mod sprite_sink;
pub mod surface;

pub use bird_painter::*;
pub use frame_renderer::*;
pub use minion_painter::*;
pub use render_thread::*;
pub use solid_painter::*;
pub use sprite_sink::*;
pub use surface::*;

/// The domain's prelude plus this crate's painters.
pub mod prelude {
    pub use crate::{
        bird_painter::*, frame_renderer::*, minion_painter::*, render_thread::*, solid_painter::*,
        sprite_sink::*, surface::*,
    };
    pub use idlewalker_domain::prelude::*;
}
