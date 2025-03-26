use std::hint::unreachable_unchecked;
use crate::shared::{pos, size};

use super::{Gui, GuiComponentView, GuiLayout, GuiAlignItems, GuiLayoutOrigin, GuiNode, ItemsDirection, ItemsPosition};

struct LayoutPositionParent {
    pub view: GuiComponentView,
    pub align_items: GuiAlignItems,
    pub child_offset: f32,
}

pub(super) fn layout_compute(gui: &mut Gui) {
    if gui.components.len() == 0 {
        return;
    }

    if gui.update_flags.compute_layout_positions() {
        position_pass(gui);
    }
}

//
// Sizing pass
//


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
        layout_position(gui, &mut index, &mut parent);
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

    let node = get_node(gui, i);
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
fn get_node(gui: &Gui, index: usize) -> GuiNode {
    match gui.components_nodes.get(index) {
        Some(node) => *node,
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

