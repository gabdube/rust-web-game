use crate::error::Error;
use crate::shared::{pos, size, AABB};
use super::*;

pub struct GuiBuilderStack {
    pub layout: GuiLayout,
    pub items_size: Size<f32>,
    pub children_count: u32,
    pub descendants_count: u32,
}

#[derive(Default)]
pub struct GuiBuilderData {
    pub build_stack: Vec<GuiBuilderStack>,
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
        data.build_stack.clear();
        data.root_index = u32::MAX;
        GuiBuilder {
            gui,
            data,
        }
    }

    //
    // Components
    //

    /// A container using an image as background
    pub fn simple_frame<CB: FnOnce(&mut GuiBuilder)>(
        &mut self,
        background: GuiResourceId<GuiImage>,
        callback: CB
    ) {
        let node = self.new_gui_node();
        let layout = self.next_layout();
        let index = self.gui.components.len();
        
        let container = GuiContainer {
            background,
            color: GuiColor::white()
        };

        self.gui.components.push(GuiComponent::Container(container));
        self.gui.components_nodes.push(node);
        self.gui.components_views.push(GuiComponentView::default());
        self.gui.components_layout.push(layout);

        self.push_stack(layout);

        callback(self);

        let items_params = self.pop_stack();
        let view = Self::container_view_from_layout(&layout, items_params.items_size);
        self.gui.components_nodes[index].children_count = items_params.children_count;
        self.gui.components_nodes[index].descendants_count = items_params.descendants_count;
        self.gui.components_views[index] = view;
        
        self.update_parent_items_size(view.size);
        self.update_parent_children_count(items_params.descendants_count);
        self.update_root_node();
    }

    /// An invisible container
    pub fn group<CB: FnOnce(&mut GuiBuilder)>(&mut self, callback: CB) {
        let node = self.new_gui_node();
        let layout = self.next_layout();
        let index = self.gui.components.len();

        self.gui.components.push(GuiComponent::Group);
        self.gui.components_nodes.push(node);
        self.gui.components_views.push(GuiComponentView::default());
        self.gui.components_layout.push(layout);

        self.push_stack(layout);

        callback(self);

        let items_params = self.pop_stack();
        let view = Self::container_view_from_layout(&layout, items_params.items_size);
        self.gui.components_nodes[index].children_count = items_params.children_count;
        self.gui.components_nodes[index].descendants_count = items_params.descendants_count;
        self.gui.components_views[index] = view;
        
        self.update_parent_items_size(view.size);
        self.update_parent_children_count(items_params.descendants_count);
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

        self.update_parent_items_size(component_size);
        self.update_parent_children_count(0);
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

        self.update_parent_items_size(component_size);
        self.update_parent_children_count(0);
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

    pub fn padding(&mut self, padding: GuiPadding) {
        self.data.next_layout.align_self.padding = padding;
    }

    pub fn items_align(&mut self, direction: ItemsDirection, position: ItemsPosition, alignment: ItemsAlign) {
        self.data.next_layout.align_items = GuiAlignItems {
            direction,
            alignment,
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
    pub fn dyn_image(&mut self) -> GuiResourceId<GuiImage> {
        let image_index = self.gui.images.len();
        let image_dyn_index = self.gui.dynamic_resources.len();
        self.gui.images.push(GuiImage::default());
        self.gui.dynamic_resources.push(DynamicResource::default());
        GuiResourceId::new_dyn(image_index, image_dyn_index)
    }

    pub fn static_text(&mut self, text: TextMetrics) -> GuiResourceId<TextMetrics> {
        self.gui.text.push(text);
        GuiResourceId::new(self.gui.text.len() - 1)
    }

    /// Add a dynamic text to the gui. The initial text is empty.
    pub fn dyn_static_text(&mut self) -> GuiResourceId<TextMetrics> {
        let text_index = self.gui.text.len();
        let text_dyn_index = self.gui.dynamic_resources.len();
        self.gui.text.push(TextMetrics::default());
        self.gui.dynamic_resources.push(DynamicResource::default());
        GuiResourceId::new_dyn(text_index, text_dyn_index)
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

    fn push_stack(&mut self, layout: GuiLayout) {
        self.data.build_stack.push(GuiBuilderStack { 
            layout,
            items_size: size(0.0, 0.0),
            children_count: 0,
            descendants_count: 0
        })
    }

    fn pop_stack(&mut self) -> GuiBuilderStack {
        match self.data.build_stack.pop() {
            Some(stack) => stack,
            _ => unsafe { std::hint::unreachable_unchecked() }
        }
    }

    fn update_parent_items_size(&mut self, child_size: Size<f32>) {
        let build = match self.data.build_stack.last_mut() {
            Some(build) => build,
            None => { return; }
        };

        match build.layout.align_items.direction {
            ItemsDirection::Column => {
                build.items_size.width = f32::max(build.items_size.width, child_size.width);
                build.items_size.height += child_size.height;
            },
            ItemsDirection::Row => {
                build.items_size.width += child_size.width;
                build.items_size.height = f32::max(build.items_size.height, child_size.height);
            }
        }
    }

    fn update_parent_children_count(&mut self, descendants_count: u32) {
        if let Some(build_stack) = self.data.build_stack.last_mut() {
            build_stack.children_count += 1;
            build_stack.descendants_count += 1 + descendants_count;
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
        if self.data.build_stack.is_empty() {
            self.data.root_index = u32::MAX;
        }
    }
}
