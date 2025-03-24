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

use crate::assets::Assets;
use crate::error::Error;

pub struct Gui {
    pub fonts: Vec<GuiFont>,
    pub text: Vec<GuiStaticText>,
    pub components: Vec<GuiComponent>,
    pub needs_sync: bool,
}

impl Gui {

    pub fn build<CB: FnOnce(&mut GuiBuilder)>(&mut self, assets: &Assets, callback: CB) -> Result<(), Error> {
        let mut builder = GuiBuilder::new(self, assets,);
        callback(&mut builder);

        if let Some(error) = builder.error {
            return Err(error);
        }

        self.needs_sync = true;
        
        Ok(())
    }

}

impl Default for Gui {

    fn default() -> Self {
        Gui {
            fonts: Vec::with_capacity(2),
            text: Vec::with_capacity(16),
            components: Vec::with_capacity(16),
            needs_sync: false,
        }
    }

}

impl crate::store::SaveAndLoad for Gui {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.fonts);
        writer.save_slice(&self.text);
        writer.write_slice(&self.components);
        writer.write_u32(self.needs_sync as u32);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        Gui {
            fonts: reader.read_slice().to_vec(),
            text: reader.load_vec(),
            components: reader.read_slice().to_vec(),
            needs_sync: reader.read_u32() == 1,
        }
    }
}

