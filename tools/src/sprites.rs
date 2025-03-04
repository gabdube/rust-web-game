//! Helpers to load sprites from images
mod optimize_animation;

use png::OutputInfo;
use std::fs::File;
use crate::shared::{Rect, Size, rect, size};

pub const PIXEL_SIZE: usize = 4; // Size of rgba u8

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
    /// The sprite spans the whole image
    Simple,
    /// The sprite in a subsection of the image delimited by `Rect`
    SimpleSub(Rect),
    /// The sprite is an animation
    Animated(AnimationInfo),
}

impl SpriteInfo {
    pub const fn auto() -> Self {
        SpriteInfo::Simple
    }

    pub const fn sub(left: u32, top: u32, right: u32, bottom: u32) -> Self {
        Self::SimpleSub(rect(left, top, right, bottom))
    }

    pub const fn animated(area: Rect, frame_size: Size) -> Self {
        Self::Animated(AnimationInfo { area, frame_size })
    }
}

/// A 2D sprite data extracted from an image
#[derive(Default)]
pub struct SpriteData {
    /// Pixel data of the sprite
    pub pixels: Vec<u8>,
    /// Size of the whole sprite
    pub size: Size,
    /// Size of a single frame in an animation. For simple sprites, this is the value of `size`
    pub frame_size: Size,
}

impl SpriteData {

    /// Load sprites from file and strip empty spaces around the image
    /// Panics if it fails to load the image
    pub fn load(path: &str, sprite_info: &SpriteInfo) -> Self {
        let (bytes, image_info) = load_image_base(path);
        let mut sprite = SpriteData::default();

        match sprite_info {
            SpriteInfo::Simple => {
                let src_rect = rect(0, 0, image_info.width, image_info.height);
                optimize_simple_sprite(image_info.line_size, &src_rect, &bytes, &mut sprite.size, &mut sprite.pixels);
                sprite.frame_size = sprite.size;
            },
            SpriteInfo::SimpleSub(src_rect) => {
                optimize_simple_sprite(image_info.line_size, &src_rect, &bytes, &mut sprite.size, &mut sprite.pixels);
                sprite.frame_size = sprite.size;

            },
            SpriteInfo::Animated(animated) => {
                let mut params = optimize_animation::OptimizeAnimationParams {
                    src_line_size: image_info.line_size,
                    src_rect: animated.area,
                    src_frame_size: animated.frame_size,
                    src_bytes: &bytes,
                    optimized_size: &mut sprite.size,
                    optimized_frame_size: &mut sprite.frame_size,
                    dst_bytes: &mut sprite.pixels
                };

                optimize_animation::optimize_animation(&mut params);
            }
        }

        sprite
    }

    pub fn line_size(&self) -> usize {
        self.size.width as usize * PIXEL_SIZE
    }

    pub fn sprite_count(&self) -> usize {
        (self.size.width / self.frame_size.width) as usize
    }

}

fn load_image_base(path: &str) -> (Vec<u8>, OutputInfo) {
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            panic!("Failed to open {path:?}: {e:?}");
        }
    };

    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().unwrap();

    let mut bytes = vec![0; reader.output_buffer_size()];
    let image_info = reader.next_frame(&mut bytes).unwrap();

    match (image_info.bit_depth, image_info.color_type) {
        (png::BitDepth::Eight, png::ColorType::Rgba) => { /* OK */ },
        combined => unimplemented!("batching sprites for {:?} is not implemented", combined)
    }

    (bytes, image_info)
}

/// Optimize a sprite. Copying the image delimited by `src_rect` in `src_bytes`, into `dst_rect` and `dst_bytes`, removing the extra unused space around the pixels
fn optimize_simple_sprite(
    src_line_size: usize,
    src_rect: &Rect,
    src_bytes: &[u8],
    dst_size: &mut Size,
    dst_bytes: &mut Vec<u8>
) {
    let mut optimized_rect = Rect::default();
    optimize_sprite_rect(src_line_size, src_rect, src_bytes, &mut optimized_rect);
    optimize_sprite_copy(src_line_size, src_bytes, &mut optimized_rect, dst_bytes);
    *dst_size = size(optimized_rect.width(), optimized_rect.height());
}

fn optimize_sprite_rect(
    src_line_size: usize,
    src_rect: &Rect,
    src_bytes: &[u8],
    optimized_rect: &mut Rect,
) {
    let mut rect = rect(u32::MAX, u32::MAX, 0, 0);

    for y in src_rect.top..src_rect.bottom {
        for x in src_rect.left..src_rect.right {
            let [x2, y2] = [x as usize, y as usize];
            let pixel_offset = (y2 * src_line_size) + (x2 * PIXEL_SIZE) + 3;
            let a: u8 = src_bytes[pixel_offset];
            if a != 0 {
                rect.left = u32::min(rect.left, x);
                rect.right = u32::max(rect.right, x);
                rect.top = u32::min(rect.top, y);
                rect.bottom = u32::max(rect.bottom, y);
            }
        }
    }

    *optimized_rect = rect;
}

fn optimize_sprite_copy(
    src_line_size: usize,
    src_bytes: &[u8],
    optimized_rect: &Rect,
    dst_bytes: &mut Vec<u8>
) {
    let width = optimized_rect.width() as usize;
    let height = optimized_rect.height() as usize;
    *dst_bytes = Vec::with_capacity(width * height * PIXEL_SIZE);

    let top = optimized_rect.top as usize;
    let bottom = optimized_rect.bottom as usize;
    let left = optimized_rect.left as usize;
    let dst_line_size = width * PIXEL_SIZE;

    for i in top..bottom {
        let bytes_start = (i * src_line_size) + (left * PIXEL_SIZE);
        dst_bytes.extend_from_slice(&src_bytes[bytes_start..bytes_start+dst_line_size]);
    }
}
