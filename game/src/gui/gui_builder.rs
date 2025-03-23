use crate::assets::FontId;
use super::*;

pub struct GuiBuilder<'a> {
    pub(super) gui: &'a mut Gui,
}

impl<'a> GuiBuilder<'a> {

    pub fn layout(&mut self, layout: GuiLayout) {

    }

    pub fn container<CB: FnOnce(&mut GuiBuilder)>(&mut self, callback: CB) {
        callback(self);
    }

    pub fn font(&mut self, font_id: FontId, size: f32) -> GuiFontId {
        self.gui.fonts.push(GuiFont { font_id, size });
        GuiFontId((self.gui.fonts.len() - 1) as u32)
    }

    pub fn static_text(&mut self, text: &str, font: FontId, color: GuiColor) -> StaticText {

        StaticText(0)
    }

    pub fn label(&mut self, label: GuiLabel) {

    }

}
