//! The innermost ring: the characters and everything they need to live, with no OS,
//! no pixels and no files. Whatever the cast needs from the outside world it asks
//! through a port declared here (`rt::IWorldApi`, `rt::LocalClock`), and the outer
//! crates implement it.
#![allow(warnings)]
pub mod rt;
pub mod settings;

pub mod expression;
pub mod hot_corners;
pub mod look;
pub mod personality;
pub mod pose;
pub mod puff;
pub mod troupe;
pub mod walker;
pub mod walker_antics;
pub mod walker_brain;
pub mod walker_eyes;
pub mod walker_mind;
pub mod walker_style;

pub use expression::*;
pub use hot_corners::*;
pub use look::*;
pub use personality::*;
pub use pose::*;
pub use puff::*;
pub use troupe::*;
pub use walker::*;
pub use walker_antics::*;
pub use walker_brain::*;
pub use walker_eyes::*;
pub use walker_mind::*;
pub use walker_style::*;

/// Everything the translated code expects in scope.
pub mod prelude {
    pub use crate::rt::{
        self, Arr, Arr2, DateTime, Dict, IWorld, Kv, Point, Random, Rectangle, StringBuilder,
    };
    pub use crate::settings::*;
    pub use crate::{
        expression::*, hot_corners::*, look::*, personality::*, pose::*, puff::*, troupe::*,
        walker::*, walker_antics::*, walker_brain::*, walker_eyes::*, walker_mind::*,
        walker_style::*,
    };
}
