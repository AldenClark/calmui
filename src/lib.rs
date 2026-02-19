pub mod components;
pub mod contracts;
pub mod feedback;
pub mod foundation;
#[cfg(feature = "i18n")]
pub mod i18n;
pub mod icon;
pub mod id;
pub mod motion;
pub mod overlay;
pub mod prelude;
mod provider;
pub mod style;
pub mod theme;
pub mod tokens;
pub mod widgets;

#[cfg(feature = "i18n")]
pub use crate::i18n::{I18nManager, Locale};
pub use provider::CalmProvider;

#[cfg(test)]
mod test_public_api;
