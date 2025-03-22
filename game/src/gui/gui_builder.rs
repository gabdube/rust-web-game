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

    pub fn font(&mut self, font: &(), size: f32, color: GuiColor) -> FontId {
        FontId(0)
    }

    pub fn static_text(&mut self, text: &str, font: FontId) -> StaticText {
        StaticText(0)
    }

    pub fn label(&mut self, label: GuiLabel) {

    }

}
