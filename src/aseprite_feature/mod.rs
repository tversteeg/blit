mod animation;
mod slice9;

pub use animation::{Animation, AnimationBlitBuffer, AnimationStatus};
use quick_error::quick_error;

quick_error::quick_error! {
    #[derive(Debug)]
    pub enum AnimationError {
        NoFrameTagsInMetadata {
            display("no frame tags field in metadata")
        }

        NoMatchingTag {
            display("no tag found which is equal to the passed tag")
        }
    }
}
