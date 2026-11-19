//! The use cases: what IdleWalker does, independent of how it is shown or which OS it
//! runs on. Everything it needs from outside is a port in `ports`, implemented by the
//! platform crate and handed in by the composition root.
pub mod brains;
pub mod jsonc;
pub mod ports;
pub mod posh;
pub mod preferences;
pub mod session;
pub mod telegram;
pub mod updates;

pub use brains::Brains;
pub use ports::{
    BrainStore, BrightnessInfo, Desktop, Displays, Journal, Modifier, MouseButtons, Os, Power,
    PreferenceStore, ReleaseFeed, RunResult, Screen, Screenshot, WebPost, Workstation,
};
pub use posh::Posh;
pub use preferences::{Indicator, Preferences};
pub use session::{Choices, CornerFire, Session, SessionPorts, TickReport};
pub use telegram::Telegram;
pub use updates::{UpdateState, Updates};
