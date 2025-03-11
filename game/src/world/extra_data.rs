use crate::store::SaveAndLoad;

#[derive(Copy, Clone, Default)]
pub struct TreeData {
    pub last_drop_timestamp: f64,
    pub being_harvested: bool,
}

impl SaveAndLoad for TreeData {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_f64(self.last_drop_timestamp);
        writer.write_u32(self.being_harvested as u32);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let last_drop_timestamp = reader.read_f64();
        let being_harvested = reader.read_u32() == 1;
        TreeData {
            last_drop_timestamp,
            being_harvested,
        }
    }
}
