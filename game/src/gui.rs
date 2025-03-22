mod gui_base;
use gui_base::*;

mod layout;
pub use layout::GuiLayout;

mod gui_resources;
pub use gui_resources::*;

mod gui_components;
pub use gui_components::*;

mod gui_builder;
pub use gui_builder::GuiBuilder;

pub struct Gui {
 
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

        }
    }

}

impl crate::store::SaveAndLoad for Gui {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        Gui {
        }
    }
}

