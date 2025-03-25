#[derive(Copy, Clone)]
pub enum GuiLayoutOrigin {
    Auto,
    TopLeft,
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
pub struct GuiAlignItems {

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

            }
        }
    }
}
