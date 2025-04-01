//! The gameplay gui state. Shared between the `editor` state and the `gameplay` state
use crate::error::Error;
use crate::gui::{GuiImageId, GuiStaticTextId};
use crate::world::{WorldObject, WorldObjectType, StructureData, ResourceType};
use crate::DemoGameData;

#[derive(Default)]
#[derive(Copy, Clone)]
pub struct GameplayGuiBindings {
    pub selected_image: GuiImageId,
    pub selected_name1: GuiStaticTextId,
    pub selected_name2: GuiStaticTextId,
    pub details_icon1: GuiImageId,
    pub details_text1: GuiStaticTextId,
}

#[derive(Default)]
#[derive(Copy, Clone)]
pub struct DetailsFrameState {
    pub displayed_object: Option<WorldObject>
}

#[derive(Default)]
#[derive(Copy, Clone)]
pub struct GameplayGuiState {
    pub bindings: GameplayGuiBindings,
    pub details_frame: DetailsFrameState
}

impl GameplayGuiState {

    pub fn build(&mut self, data: &mut DemoGameData) -> Result<(), Error> {
        use crate::gui::*;

        let bindings = &mut self.bindings;
    
        data.gui.clear();
        data.gui.resize(data.inputs.view_size);
    
        data.gui.build(|gui| {
            let text_color = GuiColor::rgb(40, 30, 20);
            let info_panel = gui.image(data.assets.gui.info_panel);
    
            gui.origin(GuiLayoutOrigin::BottomLeft);
            gui.sizing(GuiSizing::Static { width: 450.0, height: 196.0 });
            gui.items_align(ItemsDirection::Row, ItemsPosition::Start, ItemsAlign::Center);
            gui.simple_frame(info_panel, |gui| {
                
                gui.sizing(GuiSizing::Static { width: 200.0, height: 200.0 });
                gui.items_align(ItemsDirection::Column, ItemsPosition::Center, ItemsAlign::Center);
                gui.group(|gui| {
                    bindings.selected_image = gui.dyn_image();
                    gui.image_display(GuiImageDisplay::from_image(bindings.selected_image));
    
                    gui.spacer(0.0, 10.0);
    
                    bindings.selected_name1 = gui.dyn_static_text();
                    gui.label(GuiLabel::from_static_text_and_color(bindings.selected_name1, text_color));
    
                    gui.spacer(0.0, 15.0);
    
                    bindings.selected_name2 = gui.dyn_static_text();
                    gui.label(GuiLabel::from_static_text_and_color(bindings.selected_name2, text_color));
                });
    
                gui.sizing(GuiSizing::Static { width: 250.0, height: 200.0 });
                gui.padding(GuiPadding { left: 15.0, top: 25.0 });
                gui.items_align(ItemsDirection::Column, ItemsPosition::Start, ItemsAlign::Start);
                gui.group(|gui| {
                    
                    gui.items_align(ItemsDirection::Row, ItemsPosition::Center, ItemsAlign::Center);
                    gui.group(|gui| {
                        bindings.details_icon1 = gui.dyn_image();
                        gui.image_display(GuiImageDisplay::from_image_and_scaled_width(bindings.details_icon1, 28.0));
    
                        bindings.details_text1 = gui.dyn_static_text();
                        gui.label(GuiLabel::from_static_text_and_color(bindings.details_text1, text_color));
                    });
    
                });
            });
        })
    }

    pub fn set_selected_object(&mut self, data: &mut DemoGameData, new_selected: WorldObject) {
        let gui = &mut data.gui;
        let bindings = &self.bindings;
        let font = &data.assets.fonts.roboto;

        self.details_frame.displayed_object = Some(new_selected);

        let image_asset = data.assets.object_gui_image(new_selected.ty);
        gui.set_image(bindings.selected_image, image_asset);
    
        let text = font.compute_text_metrics(new_selected.ty.name(), 26.0);
        gui.set_text(bindings.selected_name1, text);

        match new_selected.ty {
            WorldObjectType::Structure => self.select_structure(data, new_selected),
            WorldObjectType::Resource => self.select_resource(data, new_selected),
            WorldObjectType::Sheep => self.select_sheep(data, new_selected),
            WorldObjectType::Tree => self.select_tree(data, new_selected),
            _ => self.select_other(data)
        }
    }

    fn select_structure(&mut self, data: &mut DemoGameData, new_selected: WorldObject) {
        let gui = &mut data.gui;
        let font = &data.assets.fonts.roboto;
        let bindings = &self.bindings;
        match data.world.structures_data[new_selected.id as usize] {
            StructureData::GoldMine(mine_data) => {
                let text_1 = font.compute_text_metrics("Gold Mine", 22.0);
                let text_2 = font.compute_text_metrics(&format!("  {} / {}", mine_data.remaining_gold, crate::world::MAX_GOLD_MINE_AMOUNT), 28.0);

                gui.set_text(bindings.selected_name2, text_1);
                gui.set_image(self.bindings.details_icon1, data.assets.gui.gold_icon);
                gui.set_text(self.bindings.details_text1, text_2);
            },
            StructureData::Castle(castle_data) => {
                let text_1 = font.compute_text_metrics("Castle", 22.0);
                let text_2 = font.compute_text_metrics(&format!("  {} / {}", castle_data.hp, crate::world::MAX_CASTLE_HP), 28.0);

                gui.set_text(bindings.selected_name2, text_1);
                gui.set_image(self.bindings.details_icon1, data.assets.gui.life_icon);
                gui.set_text(self.bindings.details_text1, text_2);
            },
            StructureData::Tower(tower_data) => {
                let text_1 = font.compute_text_metrics("Tower", 22.0);
                let text_2 = font.compute_text_metrics(&format!("  {} / {}", tower_data.hp, crate::world::MAX_TOWER_HP), 28.0);

                gui.set_text(bindings.selected_name2, text_1);
                gui.set_image(self.bindings.details_icon1, data.assets.gui.life_icon);
                gui.set_text(self.bindings.details_text1, text_2);
            },
            StructureData::House(house_data) => {
                let text_1 = font.compute_text_metrics("House", 22.0);
                let text_2 = font.compute_text_metrics(&format!("  {} / {}", house_data.hp, crate::world::MAX_HOUSE_HP), 28.0);

                gui.set_text(bindings.selected_name2, text_1);
                gui.set_image(self.bindings.details_icon1, data.assets.gui.life_icon);
                gui.set_text(self.bindings.details_text1, text_2);
            },
        }
    }

    fn select_resource(&mut self, data: &mut DemoGameData, new_selected: WorldObject) {
        let gui = &mut data.gui;
        let font = &data.assets.fonts.roboto;
        let bindings = &self.bindings;
        
        let (image, name) = match data.world.resources_data[new_selected.id as usize].resource_type {
            ResourceType::Food => (data.assets.gui.meat_icon, "Meat"),
            ResourceType::Gold => (data.assets.gui.gold_icon, "Gold"),
            ResourceType::Wood => (data.assets.gui.wood_icon, "Wood"),
        };

        gui.set_image(bindings.selected_image, image);
        gui.set_text(bindings.selected_name2, font.compute_text_metrics(name, 22.0));
        gui.clear_image(bindings.details_icon1);
        gui.clear_text(bindings.details_text1);
    }

    fn select_tree(&mut self, data: &mut DemoGameData, new_selected: WorldObject) {
        let tree_data = data.world.trees_data[new_selected.id as usize];
        let text = data.assets.fonts.roboto.compute_text_metrics(&format!("  {} / {}", tree_data.life, crate::world::MAX_TREE_LIFE), 28.0);

        data.gui.set_image(self.bindings.details_icon1, data.assets.gui.life_icon);
        data.gui.set_text(self.bindings.details_text1, text);
        data.gui.clear_text(self.bindings.selected_name2);
    }

    fn select_sheep(&mut self, data: &mut DemoGameData, new_selected: WorldObject) {
        let sheep_data = data.world.sheeps_data[new_selected.id as usize];
        let text = data.assets.fonts.roboto.compute_text_metrics(&format!("  {} / {}", sheep_data.life, crate::world::MAX_SHEEP_LIFE), 28.0);

        data.gui.set_image(self.bindings.details_icon1, data.assets.gui.life_icon);
        data.gui.set_text(self.bindings.details_text1, text);
        data.gui.clear_text(self.bindings.selected_name2);
    }

    fn select_other(&mut self, data: &mut DemoGameData) {
        let bindings = &self.bindings;
        let gui = &mut data.gui;
        gui.clear_text(bindings.selected_name2);
        gui.clear_image(bindings.details_icon1);
        gui.clear_text(bindings.details_text1);
    }
}
