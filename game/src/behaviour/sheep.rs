pub mod idle;
pub mod sheep_move;
pub mod escaping;

use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::world::World;


#[derive(Copy, Clone)]
pub enum SheepBehaviourType {
    Dead,
    Idle { time: f64 },
    Moving { target_position: Position<f32> },
    Escaping { timestamp: f64, angle: f32, },
}

#[derive(Copy, Clone)]
pub struct SheepBehaviour {
    pub ty: SheepBehaviourType,
    pub state: BehaviourState,
}

impl SheepBehaviour {

    pub fn idle() -> Self {
        SheepBehaviour {
            ty: SheepBehaviourType::Idle { time: 0.0 },
            state: BehaviourState::Initial,
        }
    }

    pub fn dead() -> Self {
        SheepBehaviour {
            ty: SheepBehaviourType::Dead,
            state: BehaviourState::Initial,
        }
    }

    pub fn escaping() -> Self {
        SheepBehaviour {
            ty: SheepBehaviourType::Escaping { timestamp: 0.0, angle: 0.0 },
            state: BehaviourState::Initial,
        }
    }

}

pub fn strike(world: &mut World, sheep_index: usize, damage: u8) {
    let sheep_data = &mut world.sheeps_data[sheep_index];
    sheep_data.life -= u8::min(sheep_data.life, damage);

    if sheep_data.life == 0 {
        world.sheep_behaviour[sheep_index] = SheepBehaviour::dead();
        // TODO: Spawn meat
    } else {
        world.sheep_behaviour[sheep_index] = SheepBehaviour::escaping();
    }
}

pub fn dead(world: &mut World, sheep_index: usize) {
    let behaviour = &mut world.sheep_behaviour[sheep_index];
    if let BehaviourState::Initial = behaviour.state {
        world.sheeps_data[sheep_index].life = 0;
        world.sheeps[sheep_index].delete();
        behaviour.state = BehaviourState::Running(0);
    }
}

impl crate::store::SaveAndLoad for SheepBehaviour {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.save(&self.state);
        writer.save(&self.ty);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let state = reader.load();
        let ty = reader.load();
        SheepBehaviour {
            state,
            ty
        }
    }
}

impl crate::store::SaveAndLoad for SheepBehaviourType {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        match self {
            Self::Dead => { writer.write_u32(0); },
            Self::Idle { time } => {
                writer.write_u32(1);
                writer.write_f64(*time);
            },
            Self::Moving { target_position } => {
                writer.write_u32(2);
                writer.write(target_position);
            },
            Self::Escaping { timestamp, angle } => {
                writer.write_u32(3);
                writer.write_f64(*timestamp);
                writer.write_f32(*angle);
            }
        }
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let id = reader.read_u32();
        match id {
            1 => Self::Idle { time: reader.read_f64() },
            2 => Self::Moving { target_position: reader.read() },
            3 => Self::Escaping { 
                timestamp: reader.read_f64(),
                angle: reader.read_f32()
            },
            _ => Self::Dead
        }

    }
}
