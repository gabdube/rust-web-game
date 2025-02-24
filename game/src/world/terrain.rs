use crate::assets::TerrainCell;
use crate::shared::AABB;

const CHUNK_STRIDE: usize = 16;
const CHUNK_STRIDE_F: f32 = 16.0;
const TERRAIN_CELL_SIZE_PX: f32 = 64.0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TerrainChunk {
    // Chunk position [u16, u16] represented as a single u32
    pub position: u32, 
    pub cells: [[TerrainCell; CHUNK_STRIDE]; CHUNK_STRIDE]
}

impl TerrainChunk {
    pub fn new(x: u32, y: u32) -> Self {
        TerrainChunk {
            position: (y<<16) | x,
            cells: unsafe { ::std::mem::zeroed() }
        }
    }

    /// Return an [AABB] representing the chunk. Values are in pixels
    pub fn view(&self) -> AABB {
        let dimension_px = CHUNK_STRIDE_F * TERRAIN_CELL_SIZE_PX;
        let y = ((self.position >> 16) as f32) * dimension_px;
        let x = ((self.position & 0xFFFF) as f32) * dimension_px;
        AABB {
            left: x,
            top: y,
            right: x + dimension_px,
            bottom: y + dimension_px,
        }
    }
}


/// The world terrain. Data is split into 16x16 tiles chunk
pub struct Terrain {
    pub chunk_width: u32,
    pub chunk_height: u32,
    pub chunks_updates: Vec<bool>,
    pub chunks: Vec<TerrainChunk>,
}

impl Terrain {

    pub fn reset(&mut self) {
        self.chunk_width = 0;
        self.chunk_height = 0;
        self.chunks_updates.clear();
        self.chunks.clear();
    }

    /// Initialize a terrain with a size of `width` by `height` cells
    pub fn init_terrain(&mut self, width: u32, height: u32) {
        let stride = CHUNK_STRIDE as u32;
        self.chunk_width = (width+(stride-1)) / stride;
        self.chunk_height = (height+(stride-1)) / stride;

        self.chunks.clear();
        self.chunks_updates.clear();

        let chunk_count = (self.chunk_width * self.chunk_height) as usize;
        self.chunks.reserve(chunk_count);
        self.chunks_updates.reserve(chunk_count);
        
        for y in 0..self.chunk_height {
            for x in 0..self.chunk_width {
                self.chunks.push(TerrainChunk::new(x, y));
                self.chunks_updates.push(true);
            }
        }
    }

}

impl crate::store::SaveAndLoad for Terrain {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_u32(self.chunk_width);
        writer.write_u32(self.chunk_height);
        writer.write_bool_slice(&self.chunks_updates);
        writer.write_slice(&self.chunks);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        Terrain {
            chunk_width: reader.read_u32(),
            chunk_height: reader.read_u32(),
            chunks_updates: reader.read_bool_vec(),
            chunks: reader.read_slice().to_vec(),
        }
    }
}

impl Default for Terrain {
    fn default() -> Self {
        Terrain {
            chunk_width: 0,
            chunk_height: 0,
            chunks_updates: Vec::new(),
            chunks: Vec::new(),
        }
    }
}
