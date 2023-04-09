/// Simplify use of `Result` in crate.
pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Any error this library can throw.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "aseprite")]
    #[error("no frame tags found in metadata of Aseprite JSON, did you export it from Aseprite using 'Array'?")]
    NoFrameTagsInMetadata,
    #[cfg(feature = "aseprite")]
    #[error("no slices found in metadata of Aseprite JSON, did you export it from Aseprite using 'Array'?")]
    NoSlicesInMetadata,
    #[cfg(feature = "aseprite")]
    #[error("no slice key found in slice `{0}` in metadata of Aseprite JSON, did you export it from Aseprite using 'Array'?")]
    NoSliceKeyInSlice(String),
    #[cfg(feature = "aseprite")]
    #[error("no center found in first key in slice `{0}` in metadata of Aseprite JSON, did you export it from Aseprite using 'Array'?")]
    NoSliceCenterInSliceKey(String),
    #[cfg(feature = "aseprite")]
    #[error("no tag found which is equal to the tagged tag `{0}` of Aseprite JSON")]
    NoMatchingFrameTag(String),
    #[cfg(feature = "aseprite")]
    #[error("animation frame `{0}` is out of bounds")]
    FrameOutOfBounds(usize),
}
