use crate::shared::AABB;

#[derive(Debug, Copy, Clone)]
pub struct GuiColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl GuiColor {
    pub const fn white() -> Self {
        GuiColor { r: 255, g: 255, b: 255 }
    }

    pub const fn splat(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GuiOutputSprite {
    pub positions: AABB,
    pub texcoord: AABB,
    pub color: GuiColor,
    pub flags: u8,
}
