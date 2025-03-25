use crate::assets::{FontId, ComputedGlyph};
use crate::shared::{Size, AABB};

#[derive(Copy, Clone)]
pub struct GuiImageId(pub u32);

#[derive(Copy, Clone)]
pub struct GuiFontId(pub u32);

#[derive(Copy, Clone)]
pub struct GuiStaticTextId(pub u32);

#[derive(Copy, Clone)]
pub struct GuiFont {
    pub size: f32,
    pub font_id: FontId,
}

#[derive(Copy, Clone)]
pub struct GuiImage {
    pub texcoord: AABB,
}

pub struct GuiStaticText {
    pub font: GuiFont,
    pub size: Size<f32>,
    pub glyphs: Box<[ComputedGlyph]>,
}

impl crate::store::SaveAndLoad for GuiStaticText {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write(&self.font);
        writer.write(&self.size);
        writer.write_slice(&self.glyphs);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let font: GuiFont = reader.read();
        let size: Size<f32> = reader.read();
        let glyphs: Vec<ComputedGlyph> = reader.read_vec();
        GuiStaticText {
            font,
            size,
            glyphs: glyphs.into_boxed_slice()
        }
    }
}
