use crate::shared::AABB;
use super::{Gui, GuiComponent, GuiComponentView, GuiContainer, GuiImageDisplay, GuiLabel, GuiOutputSprite};

pub(super) fn generate_sprites(gui: &mut Gui) {
    if !gui.update_flags.generate_sprites() {
        return;
    }

    gui.output_sprites.clear();

    let component_count = gui.components.len();
    for i in 0..component_count {
        let view = gui.components_views[i];
        let component = gui.components[i];
        match component {
            GuiComponent::Group | GuiComponent::Spacer(_) => {},
            GuiComponent::Container(background) => { generate_container(gui, view, background); }
            GuiComponent::Label(label) => { generate_label(gui, view, label); }
            GuiComponent::ImageDisplay(image) => { generate_image_display(gui, view, image); }
        }
    }
}

fn generate_container(gui: &mut Gui, view: GuiComponentView, container: GuiContainer) {
    let positions = AABB::from_position_and_size(view.position, view.size);
    let image_index = container.background.index();
    let texcoord = gui.images[image_index].texcoord;
    let color = container.color;

    gui.output_sprites.push(GuiOutputSprite {
        positions,
        texcoord,
        color,
        flags: 0,
    });
}

fn generate_label(gui: &mut Gui, view: GuiComponentView, label: GuiLabel) {
    let text_index = label.text.index();
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

fn generate_image_display(gui: &mut Gui, view: GuiComponentView, display: GuiImageDisplay) {
    let positions = AABB::from_position_and_size(view.position, view.size);
    let image_index = display.image.index();
    let texcoord = gui.images[image_index].texcoord;
    gui.output_sprites.push(GuiOutputSprite {
        positions,
        texcoord,
        color: super::GuiColor::white(),
        flags: 0,
    });
}
