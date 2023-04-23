use std::time::Duration;

use aseprite::SpritesheetData;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    Blit, BlitBuffer, BlitOptions, Size,
};

/// The animation status as returned by the `update` function of the `Animation` struct.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnimationStatus {
    /// Animation is playing and will start at the beginning when ending.
    Repeat,
    /// Animation is playing and will stop at the end.
    Playing,
    /// Animation isn't playing.
    Stopped,
}

/// The actual animation which contains the status about which frame should be played.
#[derive(Debug, Copy, Clone)]
pub struct Animation {
    frame_start: usize,
    frame_end: usize,
    frame_current: usize,
    duration: f64,
    repeat: bool,
}

impl Animation {
    /// Start an animation with a frame range, the duration of each frame is described in the Aseprite metadata.
    pub fn start(frame_start: usize, frame_end: usize, repeat: bool) -> Self {
        Animation {
            frame_start,
            frame_end,
            repeat,

            duration: 0.0,
            frame_current: frame_start,
        }
    }

    /// Start an animation with a range as described matching with a tag in the Aseprite metadata.
    pub fn start_from_tag(buffer: &AnimationBlitBuffer, tag: String, repeat: bool) -> Result<Self> {
        // Get the frame tags from the asesprite spritesheet info or throw an error when it doesn't exist
        buffer
            .info
            .meta
            .frame_tags
            // Find the frame matching the name
            .iter()
            .find(|frame_tag| frame_tag.name == tag)
            // Convert the frame to our animation struct
            .map(|frame_tag| {
                Animation::start(frame_tag.from as usize, frame_tag.to as usize, repeat)
            })
            .ok_or(Error::NoMatchingFrameTag(tag))
    }

    /// Update the animation with the time and set the current frame to the correct one.
    pub fn update(
        &mut self,
        buffer: &AnimationBlitBuffer,
        dt: Duration,
    ) -> Result<AnimationStatus> {
        // If the animation is not repeating and already passed the end point return it as stopped.
        if !self.repeat && self.frame_current > self.frame_end {
            return Ok(AnimationStatus::Stopped);
        }

        // Convert dt to seconds
        self.duration += dt.as_secs_f64();

        // Get the current frame metadata
        let mut frame = buffer
            .info
            .frames
            .get(self.frame_current)
            .ok_or(Error::FrameOutOfBounds(self.frame_current))?;

        // Loop through the frames if the time is passed
        while self.duration > frame.duration as f64 / 1000.0 {
            self.duration -= frame.duration as f64 / 1000.0;

            self.frame_current += 1;
            if self.frame_current > self.frame_end {
                break;
            }
            frame = &buffer.info.frames[self.frame_current];
        }

        // If the duration is passed the endpoint, either loop around or return as stopped
        // depending if `repeat` is true
        while self.frame_current > self.frame_end {
            self.frame_current -= self.frame_end - self.frame_start;

            if !self.repeat {
                return Ok(AnimationStatus::Stopped);
            }
        }

        if self.repeat {
            return Ok(AnimationStatus::Repeat);
        }

        Ok(AnimationStatus::Playing)
    }
}

/// `BlitBuffer` with extra information and functions to animate a sheet.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct AnimationBlitBuffer {
    buffer: BlitBuffer,
    info: SpritesheetData,
    /// Current frame to render.
    frame: usize,
}

impl AnimationBlitBuffer {
    /// Construct a new buffer for animating a spritesheet.
    pub fn new(buffer: BlitBuffer, info: SpritesheetData) -> Self {
        AnimationBlitBuffer {
            buffer,
            info,
            frame: 0,
        }
    }

    /// Update which frame to render from the [`Animation`] struct.
    pub fn update(&mut self, animation: &Animation) {
        self.frame = animation.frame_current;
    }

    /// Update to a specific frame to render.
    pub fn set_frame(&mut self, frame: usize) {
        self.frame = frame;
    }
}

impl Blit for AnimationBlitBuffer {
    fn blit(&self, dst: &mut [u32], dst_size: Size, options: &BlitOptions) {
        let frame = &self.info.frames[self.frame];

        // Area in the source to which this frame belongs
        let frame_subrect = (
            frame.frame.x as i32,
            frame.frame.y as i32,
            frame.frame.w as i32,
            frame.frame.h as i32,
        );

        todo!()
    }
}

/// Take the sub rectangle from a rectangle.
fn subrect_from_rect(
    (rect_x, rect_y, rect_width, rect_height): (i32, i32, i32, i32),
    (subrect_x, subrect_y, subrect_width, subrect_height): (i32, i32, i32, i32),
) -> (i32, i32, i32, i32) {
    debug_assert!(rect_x >= 0);
    debug_assert!(rect_y >= 0);
    debug_assert!(rect_width >= 0);
    debug_assert!(rect_height >= 0);
    debug_assert!(subrect_x >= 0);
    debug_assert!(subrect_y >= 0);
    debug_assert!(subrect_width >= 0);
    debug_assert!(subrect_height >= 0);

    let rect_edge_x = rect_x + rect_width;
    let rect_edge_y = rect_y + rect_height;

    let subrect_x = rect_x + subrect_x;
    let subrect_y = rect_y + subrect_y;
    let subrect_edge_x = subrect_x + subrect_width;
    let subrect_edge_y = subrect_y + subrect_height;

    let x = rect_x.max(subrect_x);
    let y = rect_y.max(subrect_y);
    let edge_x = rect_edge_x.min(subrect_edge_x);
    let edge_y = rect_edge_y.min(subrect_edge_y);

    (x, y, edge_x - x, edge_y - y)
}
