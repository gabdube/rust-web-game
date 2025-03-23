#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GuiColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl GuiColor {

    pub const fn white() -> Self {
        GuiColor { r: 255, g: 255, b: 255, a: 255 }
    }

}
