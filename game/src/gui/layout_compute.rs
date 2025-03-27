use std::hint::unreachable_unchecked;
use crate::shared::{Size, pos, size};

use super::{
    Gui, GuiComponentView, GuiLayout, GuiAlignItems, GuiSizing, GuiLayoutOrigin,
    GuiNode, ItemsDirection, ItemsPosition, GuiComponent
};

struct LayoutSizingParent {
    pub align_items: GuiAlignItems,
    pub size: Size<f32>,
}

struct LayoutPositionParent {
    pub view: GuiComponentView,
    pub align_items: GuiAlignItems,
    pub child_offset: f32,
}

pub(super) fn layout_compute(gui: &mut Gui) {
    if gui.components.len() == 0 {
        return;
    }

    if gui.update_flags.compute_layout_sizes() {
        sizing_pass(gui);
    }

    if gui.update_flags.compute_layout_positions() {
        position_pass(gui);
    }
}

//
// Sizing pass
//

fn sizing_pass(gui: &mut Gui) {
    let mut parent = LayoutSizingParent {
        align_items: GuiAlignItems::default(),
        size: gui.view_size
    };

    let mut index = 0;
    while index < gui.components.len() {
        let node = gui.components_nodes[index];
        if node.dirty {
            layout_size(gui, &mut index, &mut parent);
        } else {
            index += (node.descendants_count + 1) as usize;
        }
    }
}

fn get_component_size(gui: &Gui, index: usize) -> Size<f32> {
    match get_component(gui, index) {
        GuiComponent::Container(_) => size(0.0, 0.0),
        GuiComponent::ImageDisplay(image_display) => {
            gui.images[image_display.image.index()].texcoord.size()
        },
        GuiComponent::Label(label) => {
            gui.text[label.text.index()].size
        }
    }
}

fn update_parent_size(parent: &mut LayoutSizingParent, base_size: Size<f32>) {
    match parent.align_items.direction {
        ItemsDirection::Column => {
            parent.size.width = f32::max(parent.size.width, base_size.width);
            parent.size.height += base_size.height;
        }
    }
}

fn layout_size(gui: &mut Gui, index: &mut usize, parent: &mut LayoutSizingParent) {
    let i = *index;
    *index += 1;

    let node = get_node1(gui, i);
    let mut view = get_view(gui, i);
    let base_size = get_component_size(gui, i);

    update_parent_size(parent, base_size);

    if node.children_count == 0 {
        view.size = base_size;
        set_view(gui, i, view);
        return;
    }

    let layout = get_layout(gui, i);
    let mut child_sizing = LayoutSizingParent {
        align_items: layout.align_items,
        size: size(0.0, 0.0)
    };
    for _ in 0..node.children_count {
        layout_size(gui, index, &mut child_sizing)
    }
    
    view.items_size = child_sizing.size;
    view.size = match layout.align_self.sizing {
        GuiSizing::Static { width, height } => size(width, height),
        GuiSizing::Auto => child_sizing.size
    };

    set_view(gui, i, view);
}


//
// Positioning pass
//

fn position_pass(gui: &mut Gui) {
    let mut parent = LayoutPositionParent {
        view: GuiComponentView { position: pos(0.0, 0.0), size: gui.view_size, items_size: size(0.0, 0.0) },
        align_items: GuiAlignItems::default(),
        child_offset: 0.0,
    };

    let mut index = 0;
    while index < gui.components.len() {
        let node = gui.components_nodes[index];
        if node.dirty {
            layout_position(gui, &mut index, &mut parent);
        } else {
            index += (node.descendants_count + 1) as usize;
        }
    }
}

fn layout_position(gui: &mut Gui, index: &mut usize, parent: &mut LayoutPositionParent) {
    let i = *index;
    *index += 1;

    let layout = get_layout(gui, i);
    let mut view = get_view(gui, i);

    match layout.align_self.origin {
        GuiLayoutOrigin::Auto => {
            // Auto use the parent layout to position the children
            let align_items = parent.align_items;
            match (align_items.direction, align_items.position) {
                (ItemsDirection::Column, ItemsPosition::Center) => {
                    view.position.x = parent.view.position.x + ((parent.view.size.width - view.size.width) / 2.0);
                    view.position.y = parent.view.position.y + parent.child_offset;
                    parent.child_offset += view.size.height;
                },
            }
        },
        GuiLayoutOrigin::BottomLeft => {
            view.position.x = parent.view.position.x;
            view.position.y = parent.view.position.y + (parent.view.size.height - view.size.height);
        }
    }

    set_view(gui, i, view);

    let node = get_node2(gui, i);
    if node.children_count == 0 {
        return;
    }

    let mut parent = LayoutPositionParent { view, align_items: layout.align_items, child_offset: 0.0 };
    match (layout.align_items.direction, layout.align_items.position) {
        (ItemsDirection::Column, ItemsPosition::Center) => {
            parent.child_offset = (view.size.height - view.items_size.height) / 2.0;
        }
    }

    for _ in 0..node.children_count {
        layout_position(gui, index, &mut parent);
    }
}

//
// Helpers
//

#[inline(always)]
fn get_node1(gui: &Gui, index: usize) -> GuiNode {
    match gui.components_nodes.get(index) {
        Some(node) => *node,
        None => unsafe { unreachable_unchecked() }
    }
}

// Used while positioning. Also clears the node dirty flags
#[inline(always)]
fn get_node2(gui: &mut Gui, index: usize) -> GuiNode {
    match gui.components_nodes.get_mut(index) {
        Some(node) => {
            node.dirty = false;
            *node
        },
        None => unsafe { unreachable_unchecked() }
    }
}

#[inline(always)]
fn get_layout(gui: &Gui, index: usize) -> GuiLayout {
    match gui.components_layout.get(index) {
        Some(layout) => *layout,
        None => unsafe { unreachable_unchecked() }
    }
}

#[inline(always)]
fn get_view(gui: &Gui, index: usize) -> GuiComponentView {
    match gui.components_views.get(index) {
        Some(view) => *view,
        None => unsafe { unreachable_unchecked() }
    }
}

#[inline(always)]
fn set_view(gui: &mut Gui, index: usize, new_view: GuiComponentView) {
    match gui.components_views.get_mut(index) {
        Some(view) => { *view = new_view; },
        None => unsafe { unreachable_unchecked() }
    }
}

#[inline(always)]
fn get_component(gui: &Gui, index: usize) -> GuiComponent {
    match gui.components.get(index) {
        Some(component) => *component,
        None => unsafe { unreachable_unchecked() }
    }
}

