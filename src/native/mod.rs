//! Widget logic for native plataforms.

#[cfg(feature = "hexview")]
pub mod hexview;

#[cfg(feature = "hexview")]
pub use hexview::Hexview;
