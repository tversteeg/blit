//! Blit files created in Aseprite.
//!
//! Files need to be parsed with the [`aseprite`](https://docs.rs/aseprite) crate.

mod animation;

pub use animation::{Animation, AnimationBlitBuffer, AnimationStatus};
