use crate::error::Error;

/// The type of a terrain cell
#[derive(Copy, Clone)]
pub enum TerrainCell {
    Grass = 0,
    Sand,
    Water,
    Last,
}

/// Maps [TerrainCell] to their texture coordinate in the terrain texture 
pub struct TerrainTilemap {
    pub cells_texture_coordinates: Vec<[f32; 2]>
}

impl TerrainTilemap {

    pub fn load(&mut self, csv: &str) -> Result<(), Error> {
        let mut index = 0;
        crate::shared::split_csv::<3, _>(csv, |args| {
            let x = str::parse::<f32>(args[1]).unwrap_or(0.0);
            let y = str::parse::<f32>(args[2]).unwrap_or(0.0);
            self.cells_texture_coordinates[index] = [x, y];
            index += 1;
        });

        Ok(())
    }

    pub fn get_cell_texcoord(&self, cell: TerrainCell) -> [f32; 2] {
        let index = cell as usize;
        self.cells_texture_coordinates[index]
    }

}

impl crate::store::SaveAndLoad for TerrainTilemap {

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.cells_texture_coordinates);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        TerrainTilemap {
            cells_texture_coordinates: reader.read_slice().to_vec()
        }
    }

}

impl Default for TerrainTilemap {

    fn default() -> Self {
        let default = [0.0, 0.0];
        let count = TerrainCell::Last as usize;
        TerrainTilemap {
            cells_texture_coordinates: vec![default; count]
        }
    }

}
