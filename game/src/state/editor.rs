//! Special debugging state to test features
use crate::actions::Action;
use crate::state::GameState;
use crate::world::WorldObject;
use crate::{DemoGame, pos};

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum TestId {
    None,
    PawnAi
}

impl TestId {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => TestId::PawnAi,
            _ => TestId::None,
        }
    }
}

pub struct EditorState {
    current_test: TestId,
    selected_object: Option<WorldObject>,
}

impl DemoGame {

    pub fn init_editor(&mut self, test: TestId) {
        let inner_state = EditorState {
            current_test: test,
            selected_object: None,
        };

        self.init_test_terrain();

        match test {
            TestId::None => {},
            TestId::PawnAi => {
                self.init_pawn_tests();
            }
        }

        self.state = GameState::Editor(inner_state);
    }

    pub fn editor_update(&mut self) {
        if self.inputs.left_mouse_clicked() {
            on_left_mouse(self);
        }

        if self.inputs.right_mouse_clicked() {
            on_right_mouse(self)
        }
    }

    fn init_test_terrain(&mut self) {
        self.world.reset();
        self.world.init_terrain(16, 16);
        self.output.sync_world();
    }

    fn init_pawn_tests(&mut self) {
        self.world.create_pawn(pos(100.0, 100.0), &self.assets.animations.pawn.idle);
        self.world.create_pawn(pos(100.0, 200.0), &self.assets.animations.pawn.idle);
        self.world.create_pawn(pos(100.0, 300.0), &self.assets.animations.pawn.idle);
        self.world.create_tree(pos(300.0, 250.0), &self.assets.resources.tree_idle);
    }

}

fn on_left_mouse(game: &mut DemoGame) {
    let inputs = &game.inputs;
    let state = state(&mut game.state);

    let cursor_world_position = inputs.mouse_position + game.view_offset;
    let new_selected = game.world.object_at(cursor_world_position);

    match (state.selected_object, new_selected) {
        (None, None) | (Some(_), None) => {},
        (None, Some(new)) => set_new_object_selection(game, new),
        (Some(old), Some(new)) => replace_object_selection(game, old, new),
    }
}

fn on_right_mouse(game: &mut DemoGame) {
    use crate::world::WorldObjectType;

    let state = state(&mut game.state);
    let selected_object = match state.selected_object{
        Some(obj) => obj,
        None => { return; }
    };

    let inputs = &game.inputs;
    let cursor_world_position = inputs.mouse_position + game.view_offset;
    let target_object = game.world.object_at(cursor_world_position);

    match selected_object.ty {
        WorldObjectType::Pawn => {
            if target_object.is_none() {
                let action = Action::move_to(selected_object, cursor_world_position);
                game.actions.cancel(action);
                game.actions.push(action);
            }
            
            if let Some(target_object) = target_object {
                match target_object.ty {
                    WorldObjectType::Tree => move_pawn_to_tree(game, selected_object, target_object),
                    _ => {},
                }
            }
        },
        _ => {},
    }
}

fn move_pawn_to_tree(game: &mut DemoGame, selected_object: WorldObject, target: WorldObject) {
    let pawn_x = game.world.pawns[selected_object.id as usize].position.x;
    let mut target_position = game.world.trees[target.id as usize].position;
    target_position.y += 10.0;
    if target_position.x > pawn_x {
        target_position.x -= 60.0;
    } else {
        target_position.x += 60.0;
    }

    let action = Action::move_to(selected_object, target_position);
    let action2 = Action::cut_tree(selected_object, target);
    game.actions.cancel(action);
    game.actions.push_and_queue(action, action2);
}

fn set_new_object_selection(game: &mut DemoGame, new_selection: WorldObject) {
    let state = state(&mut game.state);
    game.world.set_object_selected(new_selection, true);
    state.selected_object = Some(new_selection);
}

fn replace_object_selection(game: &mut DemoGame, old_selection: WorldObject, new_selection: WorldObject) {
    let state = state(&mut game.state);
    game.world.set_object_selected(old_selection, false);
    game.world.set_object_selected(new_selection, true);
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

