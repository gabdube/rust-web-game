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
pub struct GuiImageDisplay {
    pub image: GuiImageId,
}

impl GuiImageDisplay {

    pub fn from_image(image: GuiImageId) -> Self {
        GuiImageDisplay { image }
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
    Group,
    Label(GuiLabel),
    ImageDisplay(GuiImageDisplay),
}

#[derive(Copy, Clone)]
pub struct GuiNode {
    /// Index of the root node of this component. For root component, it is it's own node index
    pub root_index: u32,
    /// Number of direct children of the component
    pub children_count: u32,
    /// Descendants count
    pub descendants_count: u32,
    /// If the component layout needs to be recomputed
    /// Right now this is only checked for root nodes
    pub dirty: bool,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct GuiComponentView {
    /// Position of the component in the gui
    pub position: Position<f32>,
    /// Size of the component
    pub size: Size<f32>,
    /// Size of the component children
    pub items_size: Size<f32>,
}
