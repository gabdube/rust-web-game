#[derive(Copy, Clone)]
pub enum GuiLayoutOrigin {
    Auto,
    TopLeft,
    BottomLeft
}

#[derive(Copy, Clone)]
pub struct GuiAlignSelf {
    pub origin: GuiLayoutOrigin, 
}

#[derive(Copy, Clone)]
pub struct GuiAlignItems {

}

#[repr(align(4))]
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
            },
            align_items: GuiAlignItems {

            }
        }
    }
}
