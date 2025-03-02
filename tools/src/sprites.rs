//! Helpers to load sprites from images
use crate::shared::{Rect, Size};

pub struct AnimationInfo {
    /// Total area of an animation
    pub area: Rect,
    /// Size of a single frame in the animation
    pub frame_size: Size,
}

impl AnimationInfo {

    pub const fn new(area: Rect, size: Size) -> Self {
        AnimationInfo { area, frame_size: size }
    }

}

pub enum SpriteInfo {
    Animated(AnimationInfo),
    Simple(Rect)
}

pub struct SpriteData {
    pub pixel_data: Vec<u8>,
    pub info: SpriteInfo,
}
