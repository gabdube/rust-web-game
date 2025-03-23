use crate::assets::FontId;

#[derive(Copy, Clone)]
pub struct GuiFontId(pub u32);

#[derive(Copy, Clone)]
pub struct StaticText(pub u32);

#[derive(Copy, Clone)]
pub struct GuiFont {
    pub font_id: FontId,
    pub size: f32,
}
