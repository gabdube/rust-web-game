use crate::assets::{FontId, Assets, ComputedGlyph};
use crate::error::Error;
use crate::shared::{pos, size, AABB};
use super::*;

#[derive(Default)]
pub struct GuiBuilderData {
    pub children_size_stack: Vec<Size<f32>>,
    pub children_count_stack: Vec<u32>,
    pub next_layout: GuiLayout,
    pub error: Option<Error>,
}

pub struct GuiBuilder<'a> {
    pub(super) gui: &'a mut Gui,
    pub(super) data: &'a mut GuiBuilderData,
    pub(super) assets: &'a Assets,
}

impl<'a> GuiBuilder<'a> {

    pub(super) fn new(gui: &'a mut Gui, assets: &'a Assets) -> Self {
        // Allow us to store the pointer to the builder data straight into the builder struct (skipping a double indirection)
        // Safety: `gui.builder_data` must not be accessed while the GuiBuilder is instanced.
        let data = unsafe { &mut *gui.builder_data.get() };
        GuiBuilder {
            gui,
            data,
            assets,
        }
    }

    //
    // Components
    //

    pub fn container<CB: FnOnce(&mut GuiBuilder)>(
        &mut self,
        background: GuiImageId,
        color: GuiColor,
        callback: CB
    ) {
        let layout = self.next_layout();
        let index = self.gui.components.len();
        
        let container = GuiContainer {
            background,
            color
        };

        self.gui.components.push(GuiComponent::Container(container));
        self.gui.components_nodes.push(GuiNode { children_count: 0 });
        self.gui.components_views.push(Self::view_from_layout(&layout));
        self.gui.components_layout.push(layout);

        self.data.children_count_stack.push(0);
        self.data.children_size_stack.push(size(0.0, 0.0));

        callback(self);

        // Sets the children count for the container
        self.gui.components_nodes[index].children_count = self.data.children_count_stack.pop().unwrap_or(0);

        // Update the component view to match the children size
        match layout.align_self.sizing {
            GuiSizing::Auto => { 
                self.gui.components_views[index].size = self.data.children_size_stack.pop().unwrap_or_default();
            },
            GuiSizing::Static { .. } => {}
        }

        let size = self.gui.components_views[index].size;
        self.update_parent_children_size(size);
        self.update_parent_children_count();
    }

    pub fn label(&mut self, label: GuiLabel) {
        let layout = self.next_layout();
        let text = &self.gui.text[label.text.0 as usize];
        let size = text.size;

        self.gui.components.push(GuiComponent::Label(label));
        self.gui.components_nodes.push(GuiNode { children_count: 0 });
        self.gui.components_layout.push(layout);

        // TODO: layout sizing for text
        self.gui.components_views.push(GuiComponentView {
            position: pos(0.0, 0.0),
            size
        });

        self.update_parent_children_size(size);
        self.update_parent_children_count();
    }

    //
    // Layout
    //

    pub fn origin(&mut self, value: GuiLayoutOrigin) {
        self.data.next_layout.align_self.origin = value;
    }

    pub fn sizing(&mut self, sizing: GuiSizing) {
        self.data.next_layout.align_self.sizing = sizing;
    }

    //
    // Resources
    //

    pub fn image(&mut self, texcoord: AABB) -> GuiImageId {
        self.gui.images.push(GuiImage { texcoord });
        GuiImageId((self.gui.images.len() - 1) as u32)
    }

    pub fn font(&mut self, font_id: FontId, size: f32) -> GuiFontId {
        self.gui.fonts.push(GuiFont { font_id, size });
        GuiFontId((self.gui.fonts.len() - 1) as u32)
    }

    pub fn static_text(&mut self, text: &str, font: GuiFontId) -> GuiStaticTextId {
        use unicode_segmentation::UnicodeSegmentation;
        
        let font = match self.gui.fonts.get(font.0 as usize) {
            Some(font) => *font,
            None => {
                self.set_error(gui_err!("Unknown font with ID {:?} in gui", font.0));
                return GuiStaticTextId(u32::MAX)
            }
        };

        let font_asset = self.assets.get_font(font.font_id);
        let mut glyphs = Vec::with_capacity(text.len());
        let mut advance = 0.0;
        let mut max_height = 0.0;
        let mut glyph = ComputedGlyph::default();
        for g in text.graphemes(true) {
            let a = font_asset.compute_glyph(g, font.size, &mut glyph);
            glyph.position.left += advance;
            glyph.position.right += advance;
    
            advance += a;
            max_height = f32::max(max_height, glyph.position.bottom);

            glyphs.push(glyph);
        }

        let size = match text.len() {
            0 => size(0.0, 0.0),
            _ => size(glyph.position.right, max_height)
        };

        self.gui.text.push(GuiStaticText { 
            font,
            size,
            glyphs: glyphs.into_boxed_slice()
        });

        GuiStaticTextId((self.gui.text.len() - 1) as u32)
    }

    //
    // Helpers
    //

    fn set_error(&mut self, err: Error) {
        match &mut self.data.error {
            Some(error) => { error.merge(err); }
            None => { self.data.error = Some(err); }
        }
    }

    fn update_parent_children_size(&mut self, child_size: Size<f32>) {
        let children_size = match self.data.children_size_stack.last_mut() {
            Some(size) => size,
            None => { return; }
        };

        children_size.width += child_size.width;
        children_size.height = f32::max(children_size.height, child_size.height);
    }

    fn update_parent_children_count(&mut self) {
        if let Some(count) = self.data.children_count_stack.last_mut() {
            *count += 1;
        }
    }

    fn next_layout(&mut self) -> GuiLayout {
        let mut out = GuiLayout::default();
        ::std::mem::swap(&mut out, &mut self.data.next_layout);
        out
    }
    
    fn view_from_layout(layout: &GuiLayout) -> GuiComponentView {
        let size = match layout.align_self.sizing {
            GuiSizing::Static { width, height } => size(width, height),
            _ => size(0.0, 0.0)
        };

        GuiComponentView {
            position: pos(0.0, 0.0),
            size,
        }
    }

}
