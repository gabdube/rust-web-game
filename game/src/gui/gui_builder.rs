use crate::assets::{FontId, Assets, ComputedGlyph};
use crate::error::Error;
use crate::shared::{pos, size, AABB};
use super::*;

#[derive(Default)]
pub struct GuiBuilderData {
    pub layout_stack: Vec<GuiLayout>,
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
        self.gui.components_nodes.push(GuiNode::default());
        self.gui.components_views.push(GuiComponentView::default());
        self.gui.components_layout.push(layout);

        self.data.children_count_stack.push(0);
        self.data.children_size_stack.push(size(0.0, 0.0));
        self.data.layout_stack.push(layout);

        callback(self);

        self.data.layout_stack.pop();

        // Sets the children count for the container
        self.gui.components_nodes[index].children_count = self.data.children_count_stack.pop().unwrap_or(0);

        // Update the component view to match the children size if needed
        let items_size = self.data.children_size_stack.pop().unwrap_or_default();
        let view = Self::container_view_from_layout(&layout, items_size);
        self.gui.components_views[index] = view;
        
        self.update_parent_children_size(view.size);
        self.update_parent_children_count();
    }

    pub fn label(&mut self, label: GuiLabel) {
        let layout = self.next_layout();

        self.gui.components.push(GuiComponent::Label(label));
        self.gui.components_nodes.push(GuiNode::default());
        self.gui.components_layout.push(layout);

        // TODO: layout sizing for text
        let text_id = label.text.0 as usize;
        let component_size = match self.gui.text.get(text_id) {
            Some(text) => text.size,
            None => {
                self.set_error(gui_err!("Unknown text with ID {:?} in gui", text_id));
                return;
            }
        };

        self.gui.components_views.push(GuiComponentView {
            position: pos(0.0, 0.0),
            size: component_size,
            items_size: size(0.0, 0.0),
        });

        self.update_parent_children_size(component_size);
        self.update_parent_children_count();
    }

    pub fn image_display(&mut self, display: GuiImageDisplay) {
        let layout = self.next_layout();

        self.gui.components.push(GuiComponent::ImageDisplay(display));
        self.gui.components_nodes.push(GuiNode::default());
        self.gui.components_layout.push(layout);

        // TODO: layout sizing for image display
        let image_id = display.image.0 as usize;
        let component_size = match self.gui.images.get(image_id) {
            Some(image) => image.texcoord.size(),
            None => {
                self.set_error(gui_err!("Unknown image with ID {:?} in gui", image_id));
                return;
            }
        };

        self.gui.components_views.push(GuiComponentView {
            position: pos(0.0, 0.0),
            size: component_size,
            items_size: size(0.0, 0.0),
        });

        self.update_parent_children_size(component_size);
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

    pub fn child_align(&mut self, direction: ChildrenDirection, position: ChildrenPosition) {
        self.data.next_layout.align_items = GuiAlignItems {
            direction,
            position,
        };
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

        match self.data.layout_stack.last() {
            Some(layout) => match layout.align_items.direction {
                ChildrenDirection::Column => {
                    children_size.width = f32::max(children_size.width, child_size.width);
                    children_size.height += child_size.height;
                }
            },
            _ => {
                children_size.width = f32::max(children_size.width, child_size.width);
                children_size.height = f32::max(children_size.height, child_size.height);
            },
        }

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
    
    fn container_view_from_layout(layout: &GuiLayout, items_size: Size<f32>) -> GuiComponentView {
        let component_size = match layout.align_self.sizing {
            GuiSizing::Static { width, height } => size(width, height),
            GuiSizing::Auto => items_size
        };

        GuiComponentView {
            position: pos(0.0, 0.0),
            size: component_size,
            items_size
        }
    }

}
