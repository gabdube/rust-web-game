//! Special debugging state to test features
use crate::behaviour;
use crate::error::Error;
use crate::state::GameState;
use crate::world::{StructureData, WorldObject, WorldObjectType};
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

pub struct EditorState {
    current_test: TestId,
    selected_object: Option<WorldObject>,
}

pub fn init(game: &mut DemoGameData, test: TestId) -> Result<(), Error> {
    let inner_state = EditorState {
        current_test: test,
        selected_object: None,
    };

    game.init_terrain(16, 16);

    init_gui(game)?;

    match test {
        TestId::None => {},
        TestId::PawnAi => init_pawn_tests(game),
        TestId::WarriorAi => init_warrior_ai(game),
        TestId::ArcherAi => init_archer_ai(game),
    }

    game.state = GameState::Editor(inner_state);

    Ok(())
}

fn init_gui(game: &mut DemoGameData) -> Result<(), Error> {
    use crate::assets::FontId;
    use crate::gui::*;

    game.gui.build(&game.assets, |gui| {
        gui.layout(GuiLayout {  });
        gui.container(|gui| {
            let font = gui.font(FontId::Roboto, 60.0);
            let text = gui.static_text("TEST", font, GuiColor::white());
            gui.label(GuiLabel::from_static_text(text));
        });
    })?;

    Ok(())
}

fn init_pawn_tests(data: &mut DemoGameData) {
    let world = &mut data.world;
    let assets = &data.assets;

    world.create_pawn(pos(100.0, 100.0), &assets.animations.pawn.idle);
    world.create_pawn(pos(100.0, 200.0), &assets.animations.pawn.idle);
    world.create_pawn(pos(100.0, 300.0), &assets.animations.pawn.idle);
    
    world.create_tree(pos(300.0, 220.0), &assets.resources.tree_idle);
    world.create_tree(pos(380.0, 300.0), &assets.resources.tree_idle);
    world.create_tree(pos(230.0, 330.0), &assets.resources.tree_idle);
    
    world.create_gold_mine(pos(200.0, 500.0), assets.structures.gold_mine_inactive);

    create_sheeps(data);
}

fn init_warrior_ai(data: &mut DemoGameData) {
    let world = &mut data.world;
    let assets = &data.assets;

    world.create_warrior(pos(100.0, 100.0), &assets.animations.warrior.idle);
    world.create_warrior(pos(200.0, 100.0), &assets.animations.warrior.idle);
    world.create_warrior(pos(300.0, 100.0), &assets.animations.warrior.idle);

    create_sheeps(data);
}

fn init_archer_ai(data: &mut DemoGameData) {
    let world = &mut data.world;
    let assets = &data.assets;

    world.create_archer(pos(100.0, 100.0), &assets.animations.archer.idle);
    world.create_archer(pos(200.0, 100.0), &assets.animations.archer.idle);
    world.create_archer(pos(300.0, 100.0), &assets.animations.archer.idle);

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

pub fn on_left_mouse(game: &mut DemoGameData) {
    let inputs = &game.inputs;
    let state = state(&mut game.state);

    let cursor_world_position = inputs.mouse_position + game.global.view_offset;
    let new_selected = game.world.object_at(cursor_world_position);

    match (state.selected_object, new_selected) {
        (None, None) | (Some(_), None) => {},
        (None, Some(new)) => set_new_object_selection(game, new),
        (Some(old), Some(new)) => replace_object_selection(game, old, new),
    }
}

pub fn on_right_mouse(game: &mut DemoGameData) {
    let state = state(&mut game.state);
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
}

fn archer_actions(game: &mut DemoGameData, archer: WorldObject, target_object: Option<WorldObject>) {
}

fn set_new_object_selection(data: &mut DemoGameData, new_selection: WorldObject) {
    let state = state(&mut data.state);
    data.world.set_object_selected(new_selection, true);
    state.selected_object = Some(new_selection);
}

fn replace_object_selection(data: &mut DemoGameData, old_selection: WorldObject, new_selection: WorldObject) {
    let state = state(&mut data.state);
    data.world.set_object_selected(old_selection, false);
    data.world.set_object_selected(new_selection, true);
    state.selected_object = Some(new_selection);
}

fn state(state: &mut GameState) -> &mut EditorState {
    match state {
        GameState::Editor(inner) => inner,
        _ => unsafe { std::hint::unreachable_unchecked() }  // state will always be editor in this module
    }
}

impl crate::store::SaveAndLoad for EditorState {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_u32(self.current_test as u32);
        writer.save_option(&self.selected_object);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let current_test = TestId::from_u32(reader.read_u32());
        let selected_object = reader.load_option();
        
        EditorState {
            current_test,
            selected_object,
        }
    }
}

