pub mod idle;
pub mod sheep_move;
pub mod escaping;

use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::DemoGameData;


#[derive(Copy, Clone)]
pub enum SheepBehaviourType {
    Dead,
    Idle { time: f64 },
    Moving { target_position: Position<f32> },
    Escaping { target_position: Position<f32> },
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
            ty: SheepBehaviourType::Escaping { target_position: Position::default() },
            state: BehaviourState::Initial,
        }
    }

}

pub fn strike(data: &mut DemoGameData, sheep_index: usize, damage: u8) {
    let sheep_data = &mut data.world.sheeps_data[sheep_index];
    sheep_data.life -= u8::min(sheep_data.life, damage);

    if sheep_data.life == 0 {
        data.world.sheep_behaviour[sheep_index] = SheepBehaviour::dead();
        spawn_meat(data, sheep_index);
    } else {
        data.world.sheep_behaviour[sheep_index] = SheepBehaviour::escaping();
    }
}

fn spawn_meat(data: &mut DemoGameData, sheep_index: usize) {
    // Spawns three food resources around the sheep
    let spawn_pos = data.world.sheeps[sheep_index].position;
    let mut position = spawn_pos;
    let mut angle = 0.0;
    for _ in 0..3 {
        angle += f32::to_radians(fastrand::u8(120..180) as f32);
        position.x = f32::ceil(spawn_pos.x + f32::cos(angle) * 64.0);
        position.y = f32::ceil(spawn_pos.y + f32::sin(angle) * 64.0);
        crate::behaviour::spawn_resources::spawn_food(data, position);
    }
}


pub fn dead(data: &mut DemoGameData, sheep_index: usize) {
    let behaviour = &mut data.world.sheep_behaviour[sheep_index];
    if let BehaviourState::Initial = behaviour.state {
        data.world.sheeps_data[sheep_index].life = 0;
        data.world.sheeps[sheep_index].delete();
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
            Self::Escaping { target_position } => {
                writer.write_u32(3);
                writer.write(target_position);
            }
        }
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let id = reader.read_u32();
        match id {
            1 => Self::Idle { time: reader.read_f64() },
            2 => Self::Moving { target_position: reader.read() },
            3 => Self::Escaping { target_position: reader.read() },
            _ => Self::Dead
        }

    }
}
