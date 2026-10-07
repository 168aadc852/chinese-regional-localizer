mod shared {
    include!("lib.rs");
}

pub use shared::*;
pub mod user_localizer;
pub use user_localizer::{UserControlledLocalizer, UserLocalizationResult};
