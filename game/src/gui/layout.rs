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

#[derive(Copy, Clone)]
pub struct GuiAlignSelf {
    pub origin: GuiLayoutOrigin,
    pub sizing: GuiSizing,
}

#[derive(Copy, Clone)]
pub enum ItemsDirection {
    Column
}

#[derive(Copy, Clone)]
pub enum ItemsPosition {
    Center
}

#[derive(Copy, Clone)]
pub struct GuiAlignItems {
    pub direction: ItemsDirection,
    pub position: ItemsPosition,
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
            },
            align_items: GuiAlignItems {
                direction: ItemsDirection::Column,
                position: ItemsPosition::Center,
            }
        }
    }
}

impl Default for GuiAlignItems {
    fn default() -> Self {
        GuiAlignItems {
            direction: ItemsDirection::Column,
            position: ItemsPosition::Center,
        }
    }
}
