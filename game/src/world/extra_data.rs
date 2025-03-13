use crate::store::SaveAndLoad;

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

#[derive(Copy, Clone, Default)]
pub struct ResourceSpawnData {
    pub spawn_time_finished: f64,
}

impl SaveAndLoad for ResourceSpawnData {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_f64(self.spawn_time_finished);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let spawn_time_finished = reader.read_f64();
        ResourceSpawnData {
            spawn_time_finished,
        }
    }
}
