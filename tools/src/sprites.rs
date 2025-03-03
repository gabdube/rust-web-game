//! Helpers to load sprites from images
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
    Simple,
    SimpleSub(Rect),
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

#[derive(Default)]
pub struct SpriteData {
    pub pixels: Vec<u8>,
    pub area: Rect,
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
                let mut optimized_rect = Rect::default();
                optimize_simple_sprite(image_info.line_size, &src_rect, &bytes, &mut optimized_rect, &mut sprite.pixels);
                sprite.area = rect(0, 0, optimized_rect.width(), optimized_rect.height());
                sprite.frame_size = size(optimized_rect.width(), optimized_rect.height());
            },
            SpriteInfo::SimpleSub(src_rect) => {
                let mut optimized_rect = Rect::default();
                optimize_simple_sprite(image_info.line_size, &src_rect, &bytes, &mut optimized_rect, &mut sprite.pixels);
                sprite.area = rect(0, 0, optimized_rect.width(), optimized_rect.height());
                sprite.frame_size = size(optimized_rect.width(), optimized_rect.height());

            },
            SpriteInfo::Animated(animated) => {
                todo!();
            }
        }

        sprite
    }

    pub fn line_size(&self) -> usize {
        self.area.width() as usize * PIXEL_SIZE
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
    optimized_rect: &mut Rect,
    dst_bytes: &mut Vec<u8>
) {
    optimize_sprite_rect(src_line_size, src_rect, src_bytes, optimized_rect);
    optimize_sprite_copy(src_line_size, src_bytes, optimized_rect, dst_bytes);
}

fn optimize_sprite_rect(
    src_line_size: usize,
    src_rect: &Rect,
    src_bytes: &[u8],
    optimized_rect: &mut Rect,
) {
    let mut rect = Rect::default();
    rect.left = u32::MAX;
    rect.top = u32::MAX;

    for y in src_rect.top..src_rect.bottom {
        for x in src_rect.left..src_rect.right {
            let [x2, y2] = [x as usize, y as usize];
            let pixel_offset = (y2 * src_line_size) + (x2 * PIXEL_SIZE);
            let a: u8 = src_bytes[pixel_offset + 3];
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
