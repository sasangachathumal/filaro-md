//! File-access layer: the only part of the app that touches the OS
//! filesystem or native dialogs. Everything markdown-rendering or UI
//! related stays out of this module.

pub mod commands;
mod error;
mod model;
mod recent;
mod walk;

#[allow(unused_imports)]
pub use error::FileAccessError;
#[allow(unused_imports)]
pub use model::{MarkdownEntry, RecentEntry};
