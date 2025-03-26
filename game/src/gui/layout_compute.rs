use std::hint::unreachable_unchecked;
use crate::shared::pos;

use super::{Gui, GuiComponentView, GuiLayout, GuiLayoutOrigin, GuiNode};

struct LayoutPositionParent {
    pub view: GuiComponentView,
}

pub(super) fn layout_compute(gui: &mut Gui) {
    if gui.components.len() == 0 {
        return;
    }

    position_pass(gui);
}

//
// Positioning pass
//

fn position_pass(gui: &mut Gui) {
    let parent = LayoutPositionParent {
        view: GuiComponentView { position: pos(0.0, 0.0), size: gui.view_size },
    };

    let mut index = 0;
    while index < gui.components.len() {
        layout_position(gui, &mut index, &parent);
    }
}

fn layout_position(gui: &mut Gui, index: &mut usize, parent: &LayoutPositionParent) {
    let i = *index;
    let layout = get_layout(gui, i);
    let mut view = get_view(gui, i);

    match layout.align_self.origin {
        GuiLayoutOrigin::Auto | GuiLayoutOrigin::TopLeft => {
            view.position.y = parent.view.position.y;
        },
        GuiLayoutOrigin::BottomLeft => {
            view.position.y = parent.view.position.y + (parent.view.size.height - view.size.height);
        }
    }

    set_view(gui, i, view);
    *index += 1;

    let node = get_node(gui, i);
    if node.children_count > 0 {
        let parent = LayoutPositionParent {
            view,
        };
        for _ in 0..node.children_count {
            layout_position(gui, index, &parent);
        }
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

