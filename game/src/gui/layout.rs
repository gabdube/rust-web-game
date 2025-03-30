#[derive(Copy, Clone)]
pub enum GuiLayoutOrigin {
    Auto,
    BottomLeft
}

#[derive(Copy, Clone)]
pub enum GuiSizing {
    Auto,
    Static { width: f32, height: f32 }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct GuiPadding {
    pub left: f32,
    pub top: f32,
}

#[derive(Copy, Clone)]
pub struct GuiAlignSelf {
    pub origin: GuiLayoutOrigin,
    pub sizing: GuiSizing,
    pub padding: GuiPadding,
}

#[derive(Copy, Clone)]
pub enum ItemsDirection {
    Column,
    Row
}

#[derive(Copy, Clone)]
pub enum ItemsAlign {
    Start,
    Center,
}

#[derive(Copy, Clone)]
pub enum ItemsPosition {
    Start,
    Center,
}

#[derive(Copy, Clone)]
pub struct GuiAlignItems {
    pub direction: ItemsDirection,
    pub alignment: ItemsAlign,
    pub position: ItemsPosition
}


#[derive(Copy, Clone)]
pub struct GuiLayout {
    pub align_self: GuiAlignSelf,
    pub align_items: GuiAlignItems,
}

impl Default for GuiLayout {
    fn default() -> Self {
        GuiLayout {
            align_self: GuiAlignSelf { 
                origin: GuiLayoutOrigin::Auto,
                sizing: GuiSizing::Auto,
                padding: GuiPadding::default(),
            },
            align_items: GuiAlignItems {
                direction: ItemsDirection::Column,
                alignment: ItemsAlign::Center,
                position: ItemsPosition::Center,
            }
        }
    }
}

impl Default for GuiAlignItems {
    fn default() -> Self {
        GuiAlignItems {
            direction: ItemsDirection::Column,
            alignment: ItemsAlign::Center,
            position: ItemsPosition::Center,
        }
    }
}
