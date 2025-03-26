mod gui_base;
pub use gui_base::*;

mod layout;
pub use layout::*;

mod gui_resources;
pub use gui_resources::*;

mod gui_components;
pub use gui_components::*;

mod gui_builder;
pub use gui_builder::{GuiBuilder, GuiBuilderData};

mod layout_compute;
mod generate_sprites;

use std::cell::UnsafeCell;
use crate::assets::TextMetrics;
use crate::error::Error;
use crate::shared::Size;

struct GuiUpdateFlags(u8);
impl GuiUpdateFlags {
    const ALL: u8 = 0b011;
    const GENERATE_SPRITES: u8          = 0b001;
    const COMPUTE_LAYOUT_POSITIONS: u8  = 0b010;
    pub fn set(&mut self, flags: u8) { self.0 = flags; }
    pub fn clear(&mut self) { self.0 = 0; }
    pub fn generate_sprites(&self) -> bool { self.0 & Self::GENERATE_SPRITES > 0 }
    pub fn compute_layout_positions(&self) -> bool { self.0 & Self::COMPUTE_LAYOUT_POSITIONS > 0 }
}

pub struct Gui {
    builder_data: UnsafeCell<GuiBuilderData>,

    images: Vec<GuiImage>,
    text: Vec<TextMetrics>,
    dynamic_resources: Vec<DynamicResource>,

    components_nodes: Vec<GuiNode>,
    components_views: Vec<GuiComponentView>, 
    components_layout: Vec<GuiLayout>,
    components: Vec<GuiComponent>,

    output_sprites: Vec<GuiOutputSprite>,

    view_size: Size<f32>,
    update_flags: GuiUpdateFlags,
}

impl Gui {

    pub fn build<CB: FnOnce(&mut GuiBuilder)>(&mut self, callback: CB) -> Result<(), Error> {
        self.clear();
        
        let mut builder = GuiBuilder::new(self);
        callback(&mut builder);
        drop(builder);

        let builder_data = self.builder_data.get_mut();
        if let Some(error) = builder_data.error.take() {
            self.clear();
            return Err(error);
        }

        self.update_flags.set(GuiUpdateFlags::ALL);
        
        Ok(())
    }

    pub fn build_sprites(&mut self) {
        layout_compute::layout_compute(self);
        generate_sprites::generate_sprites(self);
        self.update_flags.clear();
    }

    pub fn needs_sync(&self) -> bool {
        self.update_flags.0 > 0
    }

    pub fn sprites(&self) -> &[GuiOutputSprite] {
        &self.output_sprites
    }

    pub fn clear(&mut self) {
        self.images.clear();
        self.text.clear();
        self.dynamic_resources.clear();
        self.components.clear();
        self.components_views.clear();
        self.components_nodes.clear();
        self.components_layout.clear();
        self.output_sprites.clear();
        self.update_flags.set(GuiUpdateFlags::ALL);
    }

    pub fn resize(&mut self, view_size: Size<f32>) {
        self.view_size = view_size;
        if self.components.len() > 0 {
            layout_compute::layout_compute(self);
            generate_sprites::generate_sprites(self);
            self.update_flags.set(GuiUpdateFlags::ALL);
        }
    }

    pub fn set_image(&mut self, image_id: GuiResourceId<GuiImage>, image: GuiImage) {
        if !image_id.is_dyn() {
            return;
        }

        let image_index = image_id.index();
        let dyn_index = image_id.dyn_index();
        if image_index >= self.images.len() || dyn_index >= self.dynamic_resources.len() {
            return;
        }

        // Todo layout update optimization
        self.images[image_index] = image;

        self.update_flags.set(GuiUpdateFlags::ALL);
    }

}

 impl Default for Gui {

    fn default() -> Self {
        Gui {
            builder_data: UnsafeCell::new(GuiBuilderData::default()),

            images: Vec::with_capacity(16),
            text: Vec::with_capacity(16),
            dynamic_resources: Vec::with_capacity(8),

            components_nodes: Vec::with_capacity(16),
            components_views: Vec::with_capacity(16),
            components_layout: Vec::with_capacity(16),
            components: Vec::with_capacity(16),

            output_sprites: Vec::with_capacity(64),
    
            view_size: Size::default(),
            update_flags: GuiUpdateFlags(0),
        }
    }

}

impl crate::store::SaveAndLoad for Gui {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.images);
        writer.save_slice(&self.text);
        writer.save_slice(&self.dynamic_resources);
        writer.write_slice(&self.components_nodes);
        writer.write_slice(&self.components_views);
        writer.write_slice(&self.components_layout);
        writer.write_slice(&self.components);
        writer.write_slice(&self.output_sprites);
        writer.write(&self.view_size);
        writer.write_u32(self.update_flags.0 as u32);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let builder_data = UnsafeCell::new(GuiBuilderData::default());
        Gui {
            builder_data,
            images: reader.read_vec(),
            text: reader.load_vec(),
            dynamic_resources: reader.load_vec(),
            components_nodes: reader.read_vec(),
            components_views: reader.read_vec(),
            components_layout: reader.read_vec(),
            components: reader.read_vec(),
            output_sprites: reader.read_vec(),
            view_size: reader.read(),
            update_flags: GuiUpdateFlags(reader.read_u32() as u8),
        }
    }
}

