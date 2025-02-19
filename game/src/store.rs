mod reader;
pub use reader::SaveFileReader;

mod writer;
pub use writer::SaveFileWriter;

const MAGIC: u32 = 0x6FAA7601;
const ALIGN: usize = size_of::<u32>();

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SaveFileHeader {
    pub magic: u32,
    pub size: u32,
}

impl SaveFileHeader {
    pub fn new() -> Self {
        SaveFileHeader {
            magic: MAGIC,
            size: 0,
        }
    }
}

pub trait SaveAndLoad {
    fn save(&self, writer: &mut SaveFileWriter);
    fn load(reader: &mut SaveFileReader) -> Self;
}
