use super::{Gui, GuiOutputSprite, GuiComponent, GuiLabel};

pub(super) fn generate_sprites(gui: &mut Gui) {
    gui.output_sprites.clear();

    let component_count = gui.components.len();

    for i in 0..component_count {
        let component = gui.components[i];
        match component {
            GuiComponent::Label(label) => { generate_label(gui, label) }
        }
    }
}

fn generate_label(gui: &mut Gui, label: GuiLabel) {
    let text_index = label.text.0 as usize;
    let text = &gui.text[text_index];

    for glyph in text.glyphs.iter() {
        gui.output_sprites.push(GuiOutputSprite {
            positions: glyph.position,
            texcoord: glyph.texcoord,
        });
    }
}
