mod gui_base;
pub use gui_base::*;

mod layout;
pub use layout::GuiLayout;

mod gui_resources;
pub use gui_resources::*;

mod gui_components;
pub use gui_components::*;

mod gui_builder;
pub use gui_builder::GuiBuilder;

mod layout_compute;
mod generate_sprites;

use crate::assets::Assets;
use crate::error::Error;
use crate::shared::Size;

pub struct Gui {
    pub fonts: Vec<GuiFont>,
    pub text: Vec<GuiStaticText>,
    pub components: Vec<GuiComponent>,
    pub output_sprites: Vec<GuiOutputSprite>,
    pub view_size: Size<f32>,
    pub needs_sync: bool,
}

impl Gui {

    pub fn build<CB: FnOnce(&mut GuiBuilder)>(&mut self, assets: &Assets, callback: CB) -> Result<(), Error> {
        let mut builder = GuiBuilder::new(self, assets,);
        callback(&mut builder);

        if let Some(error) = builder.error {
            return Err(error);
        }

        layout_compute::layout_compute(self);
        generate_sprites::generate_sprites(self);

        self.needs_sync = true;
        
        Ok(())
    }

    pub fn clear(&mut self) {
        self.fonts.clear();
        self.text.clear();
        self.components.clear();
        self.output_sprites.clear();
        self.needs_sync = true;
    }

    pub fn resize(&mut self, view_size: Size<f32>) {
        self.view_size = view_size;
        if self.components.len() > 0 {
            layout_compute::layout_compute(self);
            generate_sprites::generate_sprites(self);
        }
    }

}

impl Default for Gui {

    fn default() -> Self {
        Gui {
            fonts: Vec::with_capacity(2),
            text: Vec::with_capacity(16),
            components: Vec::with_capacity(16),
            output_sprites: Vec::with_capacity(64),
            view_size: Size::default(),
            needs_sync: false,
        }
    }

}

impl crate::store::SaveAndLoad for Gui {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.fonts);
        writer.save_slice(&self.text);
        writer.write_slice(&self.components);
        writer.write_slice(&self.output_sprites);
        writer.write(&self.view_size);
        writer.write_u32(self.needs_sync as u32);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        Gui {
            fonts: reader.read_slice().to_vec(),
            text: reader.load_vec(),
            components: reader.read_slice().to_vec(),
            output_sprites: reader.read_slice().to_vec(),
            view_size: reader.read(),
            needs_sync: reader.read_u32() == 1,
        }
    }
}

