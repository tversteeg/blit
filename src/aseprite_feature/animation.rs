use std::{
    error::Error,
    time::Duration,
};

use aseprite::SpritesheetData;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{aseprite_feature::AnimationError, BlitBuffer};

/// The animation status as returned by the `update` function of the `Animation` struct.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnimationStatus {
    Repeat,
    Playing,
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
    /// Start an animation with a frame range, the duration of each frame is described in the
    /// aseprite metadata.
    pub fn start(frame_start: usize, frame_end: usize, repeat: bool) -> Self {
        Animation {
            frame_start,
            frame_end,
            repeat,

            duration: 0.0,
            frame_current: frame_start,
        }
    }

    /// Start an animation with a range as described matching with a tag in the aseprite metadata.
    pub fn start_from_tag(
        buffer: &AnimationBlitBuffer,
        tag: String,
        repeat: bool,
    ) -> Result<Self, AnimationError> {
        let frame_tags = match buffer.info.meta.frame_tags {
            Some(ref t) => t,
            None => return Err(AnimationError::NoFrameTagsInMetadata),
        };

        for frame in frame_tags {
            if tag == frame.name {
                return Ok(Animation::start(
                    frame.from as usize,
                    frame.to as usize,
                    repeat,
                ));
            }
        }

        Err(AnimationError::NoMatchingTag)
    }

    /// Update the animation with the time and set the current frame to the correct one.
    pub fn update(
        &mut self,
        buffer: &AnimationBlitBuffer,
        dt: Duration,
    ) -> Result<AnimationStatus, Box<dyn Error>> {
        // If the animation is not repeating and already passed the end point return it as stopped.
        if !self.repeat && self.frame_current > self.frame_end {
            return Ok(AnimationStatus::Stopped);
        }

        // Convert dt to seconds
        self.duration += dt.as_secs() as f64 + dt.subsec_nanos() as f64 * 1e-9;

        // Get the current frame metadata
        let mut frame = &buffer.info.frames[self.frame_current];

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
}

impl AnimationBlitBuffer {
    pub fn new(buffer: BlitBuffer, info: SpritesheetData) -> Self {
        AnimationBlitBuffer { buffer, info }
    }

    /// Draw one frame from the animation.
    pub fn blit_frame(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        offset: (i32, i32),
        frame: usize,
    ) -> Result<(), Box<dyn Error>> {
        let frame = &self.info.frames[frame];

        let rect = (
            frame.frame.x as i32,
            frame.frame.y as i32,
            frame.frame.w as i32,
            frame.frame.h as i32,
        );
        self.buffer.blit_rect(dst, dst_width, offset, rect);

        Ok(())
    }

    /// Draw the current frame using the animation info.
    pub fn blit(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        offset: (i32, i32),
        info: &Animation,
    ) -> Result<(), Box<dyn Error>> {
        self.blit_frame(dst, dst_width, offset, info.frame_current)
    }
}
