/*! 
    2D non tile-based pathfinding
*/ 
use std::cell::RefCell;
use crate::shared::{Position, pos};

/// Computed pathfinding data for a single unit 
#[derive(Copy, Clone)]
pub struct PathFindingData {
    pub current_position: Position<f32>,
    pub final_position: Position<f32>,
    pub graph_id: u32,
    pub graph_node_index: u32,
}

pub struct PathfindingNode {

}

/**
    A node graph of pathfinding points a character has to traverse to reach a destination
    Use interior mutability because
*/
pub struct PathfindingGraph {
    pub nodes: RefCell<Vec<Position<f32>>>,
}

/// Global pathfinding state
pub struct PathfindingState {
    pub graphs: Vec<PathfindingGraph>
}

impl PathfindingState {

    pub fn new(&mut self, start_position: Position<f32>, final_position: Position<f32>) -> PathFindingData {
        let graph_id = self.graphs.len() as u32;
        self.graphs.push(PathfindingGraph {
            nodes: RefCell::new(Vec::with_capacity(8)),
        });
        
        PathFindingData {
            current_position: start_position,
            final_position,
            graph_id,
            graph_node_index: 0,
        }
    }

    pub fn collision(&self, position: Position<f32>) -> bool {
        false
    }

    fn generate_nodes(&self, nodes: &mut Vec<Position<f32>>, path_data: &mut PathFindingData, step: f32) {
        // let mut next_position = path_data.current_position;
        // let end_position = path_data.final_position;       
        // let angle = f32::atan2(end_position.y - next_position.y, end_position.x - next_position.x);
        // while next_position != end_position {
        //     next_position = pos(
        //         next_position.x + (step * f32::cos(angle)),
        //         next_position.y + (step * f32::sin(angle)),
        //     );

        //     if self.collision(next_position) {
        //         nodes.push(next_position);
        //         todo!("Handle collisions");
        //     }

        //     if f32::abs(next_position.x - end_position.x) < step {
        //         next_position.x = end_position.x;
        //     }

        //     if f32::abs(next_position.y - end_position.y) < step {
        //         next_position.y = end_position.y;
        //     }
        // }
    }

    /**
        Builds the pathfinding graph that span from `current_position` to `final_position`

        This algorithm builds a graph of vector describing the possible movements of the character
        and saves the shortest path. Current pathfinding state is cached in `PathFindingData` for quicker
        access during behaviour evaluation.
    */
    pub fn build_graph(&self, path_data: &mut PathFindingData, step: f32) {
        let graph = &self.graphs[path_data.graph_id as usize];
        let mut nodes = graph.nodes.borrow_mut();

        nodes.push(path_data.current_position);
        self.generate_nodes(&mut nodes, path_data, step);
        nodes.push(path_data.final_position);
    }

    /**
        Store the next values of the pathfinding graph in the path data
    */
    pub fn compute_path(&self, path_data: &mut PathFindingData) {
        let graph = &self.graphs[path_data.graph_id as usize];
        let nodes = graph.nodes.borrow();

        let next_node_index = usize::min(nodes.len(), (path_data.graph_node_index + 1) as usize);
        let position = nodes[next_node_index];
       
        path_data.current_position = position;
        path_data.graph_node_index = next_node_index as u32;
    }

}

impl Default for PathfindingState {
    fn default() -> Self {
        PathfindingState {
            graphs: Vec::with_capacity(32)
        }
    }
}

impl crate::store::SaveAndLoad for PathfindingState {
    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let graphs = reader.load_vec();
        PathfindingState {
            graphs
        }
    }

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.save_slice(&self.graphs);
    }
}

impl crate::store::SaveAndLoad for PathfindingGraph {

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let nodes = reader.read_vec();
        PathfindingGraph {
            nodes: RefCell::new(nodes)
        }
    }

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        let nodes = self.nodes.borrow();
        writer.write_slice(&nodes);
    }

}
