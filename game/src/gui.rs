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

pub struct Gui {
    pub fonts: Vec<GuiFont>
}

impl Gui {

    pub fn build<CB: FnOnce(&mut GuiBuilder)>(&mut self, callback: CB) {
        let mut builder = GuiBuilder { gui: self };
        callback(&mut builder)
    }

}

impl Default for Gui {

    fn default() -> Self {
        Gui {
            fonts: Vec::with_capacity(8),
        }
    }

}

impl crate::store::SaveAndLoad for Gui {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.fonts);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        Gui {
            fonts: reader.read_slice().to_vec(),
        }
    }
}

