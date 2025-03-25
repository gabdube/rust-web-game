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
use crate::assets::Assets;
use crate::error::Error;
use crate::shared::Size;

pub struct Gui {
    builder_data: UnsafeCell<GuiBuilderData>,

    pub images: Vec<GuiImage>,
    pub fonts: Vec<GuiFont>,
    pub text: Vec<GuiStaticText>,

    pub components_nodes: Vec<GuiNode>,
    pub components_views: Vec<GuiComponentView>, 
    pub components_layout: Vec<GuiLayout>,
    pub components: Vec<GuiComponent>,

    pub output_sprites: Vec<GuiOutputSprite>,

    pub view_size: Size<f32>,
    pub needs_sync: bool,
}

impl Gui {

    pub fn build<CB: FnOnce(&mut GuiBuilder)>(&mut self, assets: &Assets, callback: CB) -> Result<(), Error> {
        let mut builder = GuiBuilder::new(self, assets);
        callback(&mut builder);
        drop(builder);

        let builder_data = self.builder_data.get_mut();
        if let Some(error) = builder_data.error.take() {
            return Err(error);
        }

        layout_compute::layout_compute(self);
        generate_sprites::generate_sprites(self);

        self.needs_sync = true;
        
        Ok(())
    }

    pub fn clear(&mut self) {
        self.images.clear();
        self.fonts.clear();
        self.text.clear();
        self.components.clear();
        self.components_views.clear();
        self.components_nodes.clear();
        self.components_layout.clear();
        self.output_sprites.clear();
        self.needs_sync = true;
    }

    pub fn resize(&mut self, view_size: Size<f32>) {
        self.view_size = view_size;
        if self.components.len() > 0 {
            layout_compute::layout_compute(self);
            generate_sprites::generate_sprites(self);
            self.needs_sync = true;
        }
    }

}

impl Default for Gui {

    fn default() -> Self {
        Gui {
            builder_data: UnsafeCell::new(GuiBuilderData::default()),

            images: Vec::with_capacity(16),
            fonts: Vec::with_capacity(2),
            text: Vec::with_capacity(16),

            components_nodes: Vec::with_capacity(16),
            components_views: Vec::with_capacity(16),
            components_layout: Vec::with_capacity(16),
            components: Vec::with_capacity(16),

            output_sprites: Vec::with_capacity(64),
    
            view_size: Size::default(),
            needs_sync: false,
        }
    }

}

impl crate::store::SaveAndLoad for Gui {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.images);
        writer.write_slice(&self.fonts);
        writer.save_slice(&self.text);
        writer.write_slice(&self.components_nodes);
        writer.write_slice(&self.components_views);
        writer.write_slice(&self.components_layout);
        writer.write_slice(&self.components);
        writer.write_slice(&self.output_sprites);
        writer.write(&self.view_size);
        writer.write_u32(self.needs_sync as u32);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let builder_data = UnsafeCell::new(GuiBuilderData::default());
        Gui {
            builder_data,
            images: reader.read_vec(),
            fonts: reader.read_vec(),
            text: reader.load_vec(),
            components_nodes: reader.read_vec(),
            components_views: reader.read_vec(),
            components_layout: reader.read_vec(),
            components: reader.read_vec(),
            output_sprites: reader.read_vec(),
            view_size: reader.read(),
            needs_sync: reader.read_u32() == 1,
        }
    }
}

