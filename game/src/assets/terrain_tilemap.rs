/// The type of a terrain cell
#[derive(Copy, Clone)]
pub enum TerrainCell {
    Grass = 0,
    Sand,
    Water,
}

/// Holds the uv coordinates of the terrain tiles type
pub struct TerrainTilemap {

}

impl TerrainTilemap {

    pub fn get_cell_texcoord(&self, cell: TerrainCell) -> [f32; 2] {
        [0.0, 0.0]
    }

}

impl Default for TerrainTilemap {

    fn default() -> Self {
        TerrainTilemap {

        }
    }

}
