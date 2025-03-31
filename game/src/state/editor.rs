//! Special debugging state to test features
use crate::behaviour;
use crate::error::Error;
use crate::gui::{GuiImageId, GuiStaticTextId};
use crate::state::GameState;
use crate::world::{ResourceType, StructureData, WorldObject, WorldObjectType};
use crate::{DemoGameData, pos};

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum TestId {
    None,
    PawnAi,
    WarriorAi,
    ArcherAi,
}

impl TestId {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => TestId::PawnAi,
            2 => TestId::WarriorAi,
            3 => TestId::ArcherAi,
            _ => TestId::None,
        }
    }
}

#[derive(Default)]
#[derive(Copy, Clone)]
pub struct EditorStateGuiBindings {
    selected_image: GuiImageId,
    selected_name1: GuiStaticTextId,
    selected_name2: GuiStaticTextId,
    selected_extra_text: GuiStaticTextId,

    details_icon1: GuiImageId,
    details_text1: GuiStaticTextId,
}

pub struct EditorState {
    current_test: TestId,
    selected_object: Option<WorldObject>,
    gui_bindings: Box<EditorStateGuiBindings>
}

//
// Init
//

pub fn init(game: &mut DemoGameData, test: TestId) -> Result<(), Error> {
    let mut inner_state = EditorState {
        current_test: test,
        selected_object: None,
        gui_bindings: Box::default(),
    };

    game.init_terrain(16, 16);

    init_gui(game, &mut inner_state)?;

    match test {
        TestId::None => {},
        TestId::PawnAi => init_pawn_tests(game),
        TestId::WarriorAi => init_warrior_ai(game),
        TestId::ArcherAi => init_archer_ai(game),
    }

    game.state = GameState::Editor(inner_state);

    Ok(())
}

fn init_gui(game: &mut DemoGameData, state: &mut EditorState) -> Result<(), Error> {
    use crate::gui::*;

    let bindings = &mut state.gui_bindings;

    game.gui.clear();
    game.gui.resize(game.inputs.view_size);

    game.gui.build(|gui| {
        let text_color = GuiColor::rgb(40, 30, 20);
        let info_panel = gui.image(game.assets.gui.info_panel);

        gui.origin(GuiLayoutOrigin::BottomLeft);
        gui.sizing(GuiSizing::Static { width: 450.0, height: 196.0 });
        gui.items_align(ItemsDirection::Row, ItemsPosition::Start, ItemsAlign::Center);
        gui.simple_frame(info_panel, |gui| {
            
            gui.sizing(GuiSizing::Static { width: 200.0, height: 200.0 });
            gui.items_align(ItemsDirection::Column, ItemsPosition::Center, ItemsAlign::Center);
            gui.group(|gui| {
                bindings.selected_image = gui.dyn_image();
                gui.image_display(GuiImageDisplay::from_image(bindings.selected_image));

                gui.spacer(0.0, 5.0);

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
                    let image = gui.image(game.assets.gui.life_icon);
                    gui.image_display(GuiImageDisplay::from_image_and_scaled_width(image, 24.0));

                    let text = gui.static_text(game.assets.fonts.roboto.compute_text_metrics("  10 / 10", 24.0));
                    gui.label(GuiLabel::from_static_text_and_color(text, text_color));
                });

            });
        });
    })?;

    Ok(())
}

fn init_pawn_tests(data: &mut DemoGameData) {
    let world = &mut data.world;
    let assets = &data.assets;

    world.create_pawn(pos(100.0, 100.0));
    world.create_pawn(pos(100.0, 200.0));
    world.create_pawn(pos(100.0, 300.0));
    
    world.create_tree(pos(300.0, 220.0), &assets.resources.tree_idle);
    world.create_tree(pos(380.0, 300.0), &assets.resources.tree_idle);
    world.create_tree(pos(230.0, 330.0), &assets.resources.tree_idle);
    
    world.create_gold_mine(pos(200.0, 500.0), assets.structures.gold_mine_inactive);

    create_sheeps(data);
}

fn init_warrior_ai(data: &mut DemoGameData) {
    let world = &mut data.world;

    world.create_warrior(pos(100.0, 100.0));
    world.create_warrior(pos(200.0, 100.0));
    world.create_warrior(pos(300.0, 100.0));

    create_sheeps(data);
}

fn init_archer_ai(data: &mut DemoGameData) {
    let world = &mut data.world;
    world.create_archer(pos(100.0, 100.0));
    world.create_archer(pos(200.0, 100.0));
    world.create_archer(pos(300.0, 100.0));

    create_sheeps(data);
}

fn create_sheeps(data: &mut DemoGameData) {
    let world = &mut data.world;
    let assets = &data.assets;

    world.create_sheep(pos(550.0, 170.0), &assets.animations.sheep.idle);
    world.create_sheep(pos(590.0, 210.0), &assets.animations.sheep.idle);
    world.create_sheep(pos(520.0, 240.0), &assets.animations.sheep.idle);
    world.create_sheep(pos(490.0, 190.0), &assets.animations.sheep.idle);
}

//
// On state events
//

pub fn on_resized(game: &mut DemoGameData) {
    game.gui.resize(game.inputs.view_size);
}

pub fn on_left_mouse(game: &mut DemoGameData) {
    let inputs = &game.inputs;
    let state = get_state(&mut game.state);

    let cursor_world_position = inputs.mouse_position + game.global.view_offset;
    let new_selected = game.world.object_at(cursor_world_position);

    match (state.selected_object, new_selected) {
        (None, None) | (Some(_), None) => {},
        (None, Some(new)) => set_new_object_selection(game, new),
        (Some(old), Some(new)) => replace_object_selection(game, old, new),
    }

    if new_selected.is_some() {
        update_selected_gui_state(game);
    }
}

pub fn on_right_mouse(game: &mut DemoGameData) {
    let state = get_state(&mut game.state);
    let selected_object = match state.selected_object {
        Some(obj) => obj,
        None => { return; }
    };

    let cursor_world_position = game.inputs.mouse_position + game.global.view_offset;
    let target_object = game.world.object_at(cursor_world_position);

    match selected_object.ty {
        WorldObjectType::Pawn => pawn_actions(game, selected_object, target_object),
        WorldObjectType::Warrior => warrior_actions(game, selected_object, target_object),
        WorldObjectType::Archer => archer_actions(game, selected_object, target_object),
        _ => {},
    }
}

fn pawn_actions(game: &mut DemoGameData, pawn: WorldObject, target_object: Option<WorldObject>) {
    let cursor_world_position = game.inputs.mouse_position + game.global.view_offset;

    if target_object.is_none() || game.inputs.left_shift.pressed() {
        behaviour::pawn::pawn_move::new(game, pawn, cursor_world_position);
        return;
    }

    let target_object = target_object.unwrap();
    match target_object.ty {
        WorldObjectType::Tree => behaviour::pawn::harvest_wood::new(game, pawn, target_object),
        WorldObjectType::Resource => behaviour::pawn::grab_resource::new(game, pawn, target_object),
        WorldObjectType::Sheep => behaviour::pawn::hunt_sheep::new(game, pawn, target_object),
        WorldObjectType::Structure => {
            match game.world.structures_data[target_object.id as usize] {
                StructureData::GoldMine(_) => behaviour::pawn::harvest_gold::new(game, pawn, target_object),
            } 
        },
        _ => {},
    }
}

fn warrior_actions(game: &mut DemoGameData, warrior: WorldObject, target_object: Option<WorldObject>) {
    let cursor_world_position = game.inputs.mouse_position + game.global.view_offset;

    if target_object.is_none() || game.inputs.left_shift.pressed() {
        behaviour::warrior::warrior_move::new(game, warrior, cursor_world_position);
        return;
    }
}

fn archer_actions(game: &mut DemoGameData, archer: WorldObject, target_object: Option<WorldObject>) {
    let cursor_world_position = game.inputs.mouse_position + game.global.view_offset;

    if target_object.is_none() || game.inputs.left_shift.pressed() {
        behaviour::archer::archer_move::new(game, archer, cursor_world_position);
        return;
    }
}

fn set_new_object_selection(data: &mut DemoGameData, new_selection: WorldObject) {
    let state = get_state(&mut data.state);
    data.world.set_object_selected(new_selection, true);
    state.selected_object = Some(new_selection);
}

fn replace_object_selection(data: &mut DemoGameData, old_selection: WorldObject, new_selection: WorldObject) {
    let state = get_state(&mut data.state);
    data.world.set_object_selected(old_selection, false);
    data.world.set_object_selected(new_selection, true);
    state.selected_object = Some(new_selection);
}

fn update_selected_gui_state(game: &mut DemoGameData) {
    let state = get_state(&mut game.state);
    let bindings = &state.gui_bindings;
    let gui = &mut game.gui;
    let font = &game.assets.fonts.roboto;

    let selected = state.selected_object.unwrap();
    let image_asset = game.assets.object_gui_image(selected.ty);
    gui.set_image(bindings.selected_image, image_asset);

    let text = font.compute_text_metrics(selected.ty.name(), 26.0);
    gui.set_text(bindings.selected_name1, text);

    match selected.ty {
        WorldObjectType::Structure => {
            match game.world.structures_data[selected.id as usize] {
                StructureData::GoldMine(_) => {
                    let text = font.compute_text_metrics("Gold Mine", 22.0);
                    gui.set_text(bindings.selected_name2, text);
                }
            }
        }
        WorldObjectType::Resource => {
            let (image, name) = match game.world.resources_data[selected.id as usize].resource_type {
                ResourceType::Food => (game.assets.gui.meat_icon, "Meat"),
                ResourceType::Gold => (game.assets.gui.gold_icon, "Gold"),
                ResourceType::Wood => (game.assets.gui.wood_icon, "Wood"),
            };

            gui.set_image(bindings.selected_image, image);
            gui.set_text(bindings.selected_name2, font.compute_text_metrics(name, 22.0));
        }
        _ => {
            gui.clear_text(bindings.selected_name2);
            gui.clear_text(bindings.selected_extra_text);
        }
    }

}

fn get_state(state: &mut GameState) -> &mut EditorState {
    match state {
        GameState::Editor(inner) => inner,
        _ => unsafe { std::hint::unreachable_unchecked() }  // state will always be editor in this module
    }
}

//
// Other
//

impl crate::store::SaveAndLoad for EditorState {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_u32(self.current_test as u32);
        writer.save_option(&self.selected_object);
        writer.write(self.gui_bindings.as_ref());
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let current_test = TestId::from_u32(reader.read_u32());
        let selected_object = reader.load_option();
        let gui_bindings = Box::new(reader.read());
        
        EditorState {
            current_test,
            selected_object,
            gui_bindings
        }
    }
}

