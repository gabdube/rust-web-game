use crate::store::SaveAndLoad;
use super::ResourceType;

#[derive(Copy, Clone)]
pub struct TreeData {
    pub last_drop_timestamp: f64,
    pub life: u8,
    pub being_harvested: bool,
}

impl Default for TreeData {
    fn default() -> Self {
        TreeData {
            last_drop_timestamp: 0.0,
            life: 10,
            being_harvested: false,
        }
    }
}

impl SaveAndLoad for TreeData {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_f64(self.last_drop_timestamp);
        writer.write_u32(self.life as u32);
        writer.write_u32(self.being_harvested as u32);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let last_drop_timestamp = reader.read_f64();
        let life = reader.read_u32() as u8;
        let being_harvested = reader.read_u32() == 1;
        TreeData {
            last_drop_timestamp,
            life,
            being_harvested,
        }
    }
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
    pub last_hit_timestamp: f64,
    pub life: u8,
}

impl Default for SheepData {
    fn default() -> Self {
        SheepData { last_hit_timestamp: 0.0, life: 10 }
    }
}

impl SaveAndLoad for SheepData {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_f64(self.last_hit_timestamp);
        writer.write_u32(self.life as u32);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let last_hit_timestamp = reader.read_f64();
        let life = reader.read_u32() as u8;
        SheepData {
            last_hit_timestamp,
            life,
        }
    }
}



#[derive(Copy, Clone)]
pub struct StructureGoldMineData {
    pub last_drop_timestamp: f64,
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
pub enum StructureData {
    GoldMine(StructureGoldMineData)
}

impl StructureData {
    pub fn gold_mine_mut(&mut self) -> &mut StructureGoldMineData {
        match self {
            StructureData::GoldMine(data) => data,
            //_ => panic!()
        }
    }
}

impl Default for StructureGoldMineData {
    fn default() -> Self {
        StructureGoldMineData {
            last_drop_timestamp: 0.0,
            miners_ids: [u32::MAX; 3],
            miners_count: 0,
            remaining_gold: 5,
        }
    }
}

impl SaveAndLoad for StructureData {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        match self {
            Self::GoldMine(value) => {
                writer.write_u32(1);
                writer.write_f64(value.last_drop_timestamp);
                writer.write(&value.miners_ids);
                writer.write_u32(value.miners_count as u32);
                writer.write_u32(value.remaining_gold as u32);
            }
        }
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let data_id = reader.read_u32();
        match data_id {
            1 => {
                Self::GoldMine(StructureGoldMineData {
                    last_drop_timestamp: reader.read_f64(),
                    miners_ids: reader.read(),
                    miners_count: reader.read_u32() as u8,
                    remaining_gold: reader.read_u32() as u8,
                })
            },
            _ => panic!("Malformed data")
        }
    }
}
