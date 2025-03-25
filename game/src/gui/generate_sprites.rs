use crate::shared::AABB;
use super::{Gui, GuiComponent, GuiComponentView, GuiLabel, GuiOutputSprite, GuiColor};

pub(super) fn generate_sprites(gui: &mut Gui) {
    gui.output_sprites.clear();

    let component_count = gui.components.len();
    if component_count == 0 {
        return;
    }

    for i in 0..component_count {
        let view = gui.components_views[i];
        let component = gui.components[i];
        match component {
            GuiComponent::Container => { generate_container(gui, view); }
            GuiComponent::Label(label) => { generate_label(gui, view, label); }
        }
    }
}

fn generate_container(gui: &mut Gui, view: GuiComponentView) {
    let positions = AABB {
        left: view.position.x,
        top: view.position.y,
        right: view.position.x + view.size.width,
        bottom: view.position.y + view.size.height,
    };

    gui.output_sprites.push(GuiOutputSprite {
        positions,
        texcoord: AABB::default(),
        color: GuiColor::black(),
        flags: 0,
    });
}

fn generate_label(gui: &mut Gui, view: GuiComponentView, label: GuiLabel) {
    let text_index = label.text.0 as usize;
    let text = &gui.text[text_index];

    for glyph in text.glyphs.iter() {
        let mut positions = glyph.position;
        positions.offset(view.position);

        gui.output_sprites.push(GuiOutputSprite {
            positions,
            texcoord: glyph.texcoord,
            color: label.text_color,
            flags: 1,
        });
    }
}
