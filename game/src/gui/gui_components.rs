use super::{GuiBuilder, GuiStaticTextId};

#[derive(Copy, Clone)]
pub struct GuiLabel {
    pub text: GuiStaticTextId
}

impl GuiLabel {

    pub fn from_static_text(text: GuiStaticTextId) -> Self {
        GuiLabel {
            text,
        }
    }

}

#[derive(Copy, Clone)]
pub enum GuiComponent {
    Label(GuiLabel)
}
