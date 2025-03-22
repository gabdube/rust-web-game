use super::{GuiBuilder, StaticText};

pub struct GuiLabel {
    pub text: StaticText
}

impl GuiLabel {

    pub fn from_static_text(builder: &mut GuiBuilder, text: StaticText) -> Self {
        GuiLabel {
            text,
        }
    }

}
