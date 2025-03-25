use crate::shared::{Position, Size};
use super::{GuiColor, GuiImageId, GuiStaticTextId};

#[derive(Copy, Clone)]
pub struct GuiLabel {
    pub text: GuiStaticTextId,
    pub text_color: GuiColor,
}

impl GuiLabel {

    pub fn from_static_text_and_color(text: GuiStaticTextId, text_color: GuiColor) -> Self {
        GuiLabel {
            text,
            text_color,
        }
    }

}

#[derive(Copy, Clone)]
pub struct GuiContainer {
    pub background: GuiImageId,
    pub color: GuiColor,
}

#[derive(Copy, Clone)]
pub enum GuiComponent {
    Container(GuiContainer),
    Label(GuiLabel)
}

#[derive(Copy, Clone)]
pub struct GuiNode {
    pub children_count: u32,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct GuiComponentView {
    pub position: Position<f32>,
    pub size: Size<f32>,
}
