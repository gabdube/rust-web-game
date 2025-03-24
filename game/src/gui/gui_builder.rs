use crate::assets::{FontId, Assets, ComputedGlyph};
use crate::error::Error;
use crate::shared::size;
use super::*;

pub struct GuiBuilder<'a> {
    pub(super) gui: &'a mut Gui,
    pub(super) assets: &'a Assets,
    pub(super) error: Option<Error>,
}

impl<'a> GuiBuilder<'a> {

    pub(super) fn new(gui: &'a mut Gui, assets: &'a Assets) -> Self {
        GuiBuilder {
            gui,
            assets,
            error: None,
        }
    }

    pub fn layout(&mut self, layout: GuiLayout) {

    }

    pub fn container<CB: FnOnce(&mut GuiBuilder)>(&mut self, callback: CB) {
        callback(self);
    }

    pub fn font(&mut self, font_id: FontId, size: f32) -> GuiFontId {
        self.gui.fonts.push(GuiFont { font_id, size });
        GuiFontId((self.gui.fonts.len() - 1) as u32)
    }

    pub fn static_text(&mut self, text: &str, font: GuiFontId, color: GuiColor) -> GuiStaticTextId {
        use unicode_segmentation::UnicodeSegmentation;
        
        let font = match self.gui.fonts.get(font.0 as usize) {
            Some(font) => *font,
            None => {
                self.set_error(gui_err!("Unknown font with ID {:?} in gui", font.0));
                return GuiStaticTextId(u32::MAX)
            }
        };

        let font_asset = self.assets.get_font(font.font_id);
        let mut glyphs = Vec::with_capacity(text.len());
        let mut advance = 0.0;
        let mut glyph = ComputedGlyph::default();
        for g in text.graphemes(true) {
            let a = font_asset.compute_glyph(g, font.size, &mut glyph);
            glyph.position.left += advance;
            glyph.position.right += advance;
    
            advance += a;
            glyphs.push(glyph);
        }

        let size = match text.len() {
            0 => size(0.0, 0.0),
            _ => size(glyph.position.right, 0.0)
        };

        self.gui.text.push(GuiStaticText { 
            font,
            size,
            color,
            glyphs: glyphs.into_boxed_slice()
        });

        GuiStaticTextId((self.gui.text.len() - 1) as u32)
    }

    pub fn label(&mut self, label: GuiLabel) {
        self.gui.components.push(GuiComponent::Label(label));
    }

    fn set_error(&mut self, err: Error) {
        match &mut self.error {
            Some(error) => { error.merge(err); }
            None => { self.error = Some(err); }
        }
    }
}
