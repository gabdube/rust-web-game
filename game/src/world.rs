use crate::store::SaveAndLoad;
use crate::Position;

#[derive(Copy, Clone, Debug)]
pub struct Pawn {
    pub position: Position<f32>,
}

/// The game world data. Includes actors, terrain, and decorations
pub struct World {
    pub pawns: Vec<Pawn>
}

impl SaveAndLoad for World {

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.pawns);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let pawns = reader.read_slice().to_vec();
        World {
            pawns,
        }
    }

}

impl Default for World {
    fn default() -> Self {
        World {
            pawns: Vec::with_capacity(16),
        }
    }
}
