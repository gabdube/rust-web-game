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
