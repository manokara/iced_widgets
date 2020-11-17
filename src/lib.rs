//! manokara's iced widgets
//!
//! A collection of widgets for the [iced] GUI crate.
//!
//! # Widget List
//!
//! - [`Hexview`]: A widget for viewing binary data.
//!
//! [iced]: https://github.com/hecrj/iced
//! [`Hexview`]: native/hexview/struct.Hexview.html
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[cfg(not(any(feature = "hexview")))]
compile_error!("No widgets to be compiled.");

pub mod core;
pub mod graphics;
pub mod native;
pub mod style;
