//! REgroups all the world structure data into a single file

use crate::shared::Position;
use crate::store::SaveAndLoad;
use super::WorldObject;

pub const MAX_CASTLE_HP: u8 = 200;
pub const MAX_TOWER_HP: u8 = 80;
pub const MAX_HOUSE_HP: u8 = 50;
pub const MAX_GOLD_MINE_AMOUNT: u8 = 10;
pub const MAX_TREE_LIFE: u8 = 15;
pub const MAX_SHEEP_LIFE: u8 = 10;

#[derive(Copy, Clone)]
pub struct TreeData {
    pub life: u8,
    pub being_harvested: bool,
}

impl Default for TreeData {
    fn default() -> Self {
        TreeData {
            life: MAX_TREE_LIFE,
            being_harvested: false,
        }
    }
}

impl SaveAndLoad for TreeData {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_u32(self.life as u32);
        writer.write_u32(self.being_harvested as u32);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let life = reader.read_u32() as u8;
        let being_harvested = reader.read_u32() == 1;
        TreeData {
            life,
            being_harvested,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ResourceType {
    Wood,
    Food,
    Gold
}

/// Align the resource to 4 bytes to allow quick store/load
#[repr(align(4))]
#[derive(Copy, Clone)]
pub struct ResourceData {
    pub resource_type: ResourceType,
    pub grabbed: bool,
}

#[derive(Copy, Clone)]
pub struct PawnData {
    pub grabbed_resource: u32,
}

impl PawnData {
    pub fn grabbed_resource(&self) -> Option<usize> {
        match self.grabbed_resource == u32::MAX {
            true => None,
            false => Some(self.grabbed_resource as usize)
        }
    }
}

impl Default for PawnData {
    fn default() -> Self {
        PawnData {
            grabbed_resource: u32::MAX,
        }
    }
}

#[derive(Copy, Clone)]
pub struct SheepData {
    pub anchor_position: Position<f32>,
    pub life: u8,
}

impl Default for SheepData {
    fn default() -> Self {
        SheepData { anchor_position: Position::default(), life: 10 }
    }
}

#[derive(Copy, Clone)]
pub struct StructureGoldMineData {
    pub miners_ids: [u32; 3],
    pub miners_count: u8,
    pub remaining_gold: u8,
}

impl StructureGoldMineData {
    pub fn can_be_mined(&self) -> bool {
        self.miners_count < 3 && self.remaining_gold > 0
    }
}

#[derive(Copy, Clone)]
pub struct StructureCastleData {
    pub hp: u8,
    pub building: bool,
    pub destroyed: bool,
}

#[derive(Copy, Clone)]
pub struct StructureTowerData {
    pub hp: u8,
    pub building: bool,
    pub destroyed: bool,
}

#[derive(Copy, Clone)]
pub struct StructureHouseData {
    pub hp: u8,
    pub building: bool,
    pub destroyed: bool,
}

#[derive(Copy, Clone)]
pub enum StructureData {
    GoldMine(StructureGoldMineData),
    Castle(StructureCastleData),
    Tower(StructureTowerData),
    House(StructureHouseData),
}

impl StructureData {

    pub fn gold_mine(self) -> StructureGoldMineData {
        match self {
            StructureData::GoldMine(data) => data,
            _ => panic!("Structure data is not gold mine")
        }
    }
}

impl Default for StructureGoldMineData {
    fn default() -> Self {
        StructureGoldMineData {
            miners_ids: [u32::MAX; 3],
            miners_count: 0,
            remaining_gold: MAX_GOLD_MINE_AMOUNT,
        }
    }
}

#[derive(Copy, Clone)]
pub struct ArrowData {
    pub velocity: Position<f32>,
    pub target_position: Position<f32>,
    pub target_entity: WorldObject,
    pub arrow_tip_offset: Position<f32>,
}
