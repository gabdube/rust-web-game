use crate::error::Error;
use crate::shared::{pos, size, AABB};
use super::*;

#[derive(Default)]
pub struct GuiBuilderData {
    pub layout_stack: Vec<GuiLayout>,
    pub children_size_stack: Vec<Size<f32>>,
    pub children_count_stack: Vec<[u32; 2]>,
    pub next_layout: GuiLayout,
    pub error: Option<Error>,
    pub root_index: u32,
}

pub struct GuiBuilder<'a> {
    pub(super) gui: &'a mut Gui,
    pub(super) data: &'a mut GuiBuilderData,
}

impl<'a> GuiBuilder<'a> {

    pub(super) fn new(gui: &'a mut Gui) -> Self {
        // Allow us to store the pointer to the builder data straight into the builder struct (skipping a double indirection)
        // Safety: `gui.builder_data` must not be accessed while the GuiBuilder is instanced.
        let data = unsafe { &mut *gui.builder_data.get() };
        data.root_index = u32::MAX;
        GuiBuilder {
            gui,
            data,
        }
    }

    //
    // Components
    //

    pub fn container<CB: FnOnce(&mut GuiBuilder)>(
        &mut self,
        background: GuiResourceId<GuiImage>,
        color: GuiColor,
        callback: CB
    ) {
        let node = self.new_gui_node();
        let layout = self.next_layout();
        let index = self.gui.components.len();
        
        let container = GuiContainer {
            background,
            color
        };

        self.gui.components.push(GuiComponent::Container(container));
        self.gui.components_nodes.push(node);
        self.gui.components_views.push(GuiComponentView::default());
        self.gui.components_layout.push(layout);

        self.data.children_count_stack.push([0, 0]);
        self.data.children_size_stack.push(size(0.0, 0.0));
        self.data.layout_stack.push(layout);

        callback(self);

        self.data.layout_stack.pop();

        // Sets the children count for the container
        let [children_count, descendants_count] = self.children_count();
        let node = &mut self.gui.components_nodes[index];
        node.children_count = children_count;
        node.descendants_count = descendants_count;

        // Update the component view to match the children size if needed
        let items_size = self.data.children_size_stack.pop().unwrap_or_default();
        let view = Self::container_view_from_layout(&layout, items_size);
        self.gui.components_views[index] = view;
        
        self.update_parent_children_size(view.size);
        self.update_parent_children_count();
        self.update_root_node();
    }

    pub fn label(&mut self, label: GuiLabel) {
        let node = self.new_gui_node();
        let layout = self.next_layout();

        if label.text.is_dyn() {
            let dyn_resource = &mut self.gui.dynamic_resources[label.text.dyn_index()];
            dyn_resource.users.push(self.gui.components.len() as u32);
        }

        self.gui.components.push(GuiComponent::Label(label));
        self.gui.components_nodes.push(node);
        self.gui.components_layout.push(layout);

        // TODO: layout sizing for text
        let text_id = label.text.index();
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
        self.update_root_node();
    }

    pub fn image_display(&mut self, display: GuiImageDisplay) {
        let node = self.new_gui_node();
        let layout = self.next_layout();

        if display.image.is_dyn() {
            let dyn_resource = &mut self.gui.dynamic_resources[display.image.dyn_index()];
            dyn_resource.users.push(self.gui.components.len() as u32);
        }

        self.gui.components.push(GuiComponent::ImageDisplay(display));
        self.gui.components_nodes.push(node);
        self.gui.components_layout.push(layout);

        // TODO: layout sizing for image display
        let image_id = display.image.index();
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
        self.update_root_node();
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

    pub fn items_align(&mut self, direction: ItemsDirection, position: ItemsPosition) {
        self.data.next_layout.align_items = GuiAlignItems {
            direction,
            position,
        };
    }

    //
    // Resources
    //

    pub fn image(&mut self, texcoord: AABB) -> GuiResourceId<GuiImage> {
        self.gui.images.push(GuiImage { texcoord });
        GuiResourceId::new(self.gui.images.len() - 1)
    }

    /// Add a dynamic image to the gui. The initial image data is empty.
    pub fn dyn_empty_image(&mut self) -> GuiResourceId<GuiImage> {
        let image_index = self.gui.images.len();
        let image_dyn_index = self.gui.dynamic_resources.len();
        self.gui.images.push(GuiImage { texcoord: AABB::default() });
        self.gui.dynamic_resources.push(DynamicResource::default());
        GuiResourceId::new_dyn(image_index, image_dyn_index)
    }

    pub fn static_text(&mut self, text: TextMetrics) -> GuiResourceId<GuiStaticText> {
        self.gui.text.push(text);
        GuiResourceId::new(self.gui.text.len() - 1)
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
                ItemsDirection::Column => {
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
        if let Some([children_count, descendants_count]) = self.data.children_count_stack.last_mut() {
            *children_count += 1;
            *descendants_count += 1;
        }
    }

    fn children_count(&mut self) -> [u32; 2] {
        let [children_count, descendants_count] = self.data.children_count_stack.pop().unwrap_or([0, 0]);
        if let Some([_, parent_descendants_count]) = self.data.children_count_stack.last_mut() {
            *parent_descendants_count += children_count;
        }

        [children_count, descendants_count]
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

    fn new_gui_node(&mut self) -> GuiNode {
        if self.data.root_index == u32::MAX {
            self.data.root_index = self.gui.components_nodes.len() as u32;
        }
        
        GuiNode {
            root_index: self.data.root_index,
            children_count: 0,
            descendants_count: 0,
            dirty: true,
        }
    }

    fn update_root_node(&mut self) {
        if self.data.children_size_stack.is_empty() {
            self.data.root_index = u32::MAX;
        }
    }
}
