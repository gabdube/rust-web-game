pub mod warrior_move;
pub mod warrior_attack;

use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::world::WorldObject;
use crate::DemoGameData;

#[derive(Copy, Clone)]
pub enum WarriorBehaviourType {
    Idle,
    MoveTo { target_position: Position<f32> },
    Attack { target: WorldObject, timestamp1: f64, timestamp2: f64 }
}

#[derive(Copy, Clone)]
pub struct WarriorBehaviour {
    pub ty: WarriorBehaviourType,
    pub state: BehaviourState,
}

impl WarriorBehaviour {
    pub fn idle() -> Self {
        WarriorBehaviour {
            ty: WarriorBehaviourType::Idle,
            state: BehaviourState::Initial
        }
    }
}

pub fn idle(game: &mut DemoGameData, warrior_index: usize) {
    let world = &mut game.world;
    let behaviour = &mut world.warriors_behaviour[warrior_index];
    if let BehaviourState::Initial = behaviour.state {
        let warrior = &mut world.warriors[warrior_index];
        warrior.animation = game.assets.animations.warrior.idle;
        behaviour.state = BehaviourState::Running(0);
    }
}

impl crate::store::SaveAndLoad for WarriorBehaviour {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.save(&self.state);
        writer.save(&self.ty);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let state = reader.load();
        let ty = reader.load();
        WarriorBehaviour {
            state,
            ty
        }
    }
}

impl crate::store::SaveAndLoad for WarriorBehaviourType {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        match self {
            Self::Idle => {
                writer.write_u32(1);
            },
            Self::MoveTo { target_position } => {
                writer.write_u32(2);
                writer.write(target_position);
            },
            Self::Attack { target, timestamp1, timestamp2 } => {
                writer.write_u32(3);
                writer.write(target);
                writer.write_f64(*timestamp1);
                writer.write_f64(*timestamp2);
            }
        }
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let id = reader.read_u32();
        match id {
            2 => Self::MoveTo { target_position: reader.read() },
            3 => Self::Attack { target: reader.read(), timestamp1: reader.read_f64(), timestamp2: reader.read_f64() },
            _ => Self::Idle,
        }

    }
}
