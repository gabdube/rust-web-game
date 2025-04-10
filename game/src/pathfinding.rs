/*! 
    2D non tile-based pathfinding 

*/ 
use crate::shared::Position;

/// Computed pathfinding data for a single unit 
#[derive(Copy, Clone)]
pub struct PathFindingData {
    pub next_position: Position<f32>,
    pub final_position: Position<f32>,
}

/// Global pathfinding state
pub struct PathfindingState {

}

impl PathfindingState {

    pub fn new(&mut self, start_position: Position<f32>, final_position: Position<f32>) -> PathFindingData {
        PathFindingData {
            next_position: start_position,
            final_position
        }
    }

    /// Returns `true` if the path was fully computed, `false` if it was not fully computed
    pub fn compute_path(&self, path_data: &mut PathFindingData) -> bool {
        false
    }

}

impl Default for PathfindingState {
    fn default() -> Self {
        PathfindingState {

        }
    }
}

impl crate::store::SaveAndLoad for PathfindingState {
    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        PathfindingState {

        }
    }

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        
    }
}
