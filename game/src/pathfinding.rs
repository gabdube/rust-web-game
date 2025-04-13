/*! 
    2D non tile-based pathfinding
*/ 
use std::cell::RefCell;
use crate::shared::{pos, Position, AABB};

type Vec2 = [Position<f32>; 2];

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Corner {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom
}

impl Corner {
    const fn left(self) -> bool { match self { Self::LeftTop | Self::LeftBottom => true, _ => false } }
    const fn right(self) -> bool { match self { Self::RightTop | Self::RightBottom => true, _ => false } }
    const fn top(self) -> bool { match self { Self::LeftTop | Self::RightTop => true, _ => false } }
    const fn bottom(self) -> bool { match self { Self::LeftBottom | Self::RightBottom => true, _ => false } }
}

/// Computed pathfinding data for a single unit 
#[derive(Copy, Clone)]
pub struct PathFindingData {
    pub next_position: Position<f32>,
    pub graph_id: u32,
    pub graph_node_index: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct PathfindingCollision {
    pub intersection: Position<f32>,
    pub aabb: AABB,
}

/**
    A node graph of pathfinding points a character has to traverse to reach a destination
*/
pub struct PathfindingGraph {
    pub nodes: RefCell<Vec<Position<f32>>>,
    pub free: bool,
}

/// Global pathfinding state
pub struct PathfindingState {
    pub static_collisions: Vec<AABB>,
    pub navmesh: (),
    pub graphs: Vec<PathfindingGraph>
}

impl PathfindingState {

    pub fn generate_navmesh(&mut self) {

    }

    pub fn compute_new_path(&mut self, start_position: Position<f32>, final_position: Position<f32>) -> PathFindingData {
        let graph_id;
        match self.graphs.iter_mut().enumerate().find(|(_,g)| g.free ) {
            Some((index, graph)) => {
                graph_id = index as u32;
                graph.free = false;
            },
            None => {
                graph_id = self.graphs.len() as u32;
                self.graphs.push(PathfindingGraph {
                    nodes: RefCell::new(Vec::with_capacity(8)),
                    free: false,
                });
            }
        }

        self.graphs[graph_id as usize].nodes.get_mut().push(start_position);
        self.graphs[graph_id as usize].nodes.get_mut().push(final_position);

        PathFindingData {
            next_position: start_position,
            graph_id,
            graph_node_index: 0,
        }
    }

    pub fn clear(&mut self) {
        self.static_collisions.clear();

        for graph in self.graphs.iter_mut() {
            graph.nodes.get_mut().clear();
            graph.free = true;
        }
    }

    pub fn register_static_collision(&mut self, aabb: AABB) {
        self.static_collisions.push(aabb);
    }

    pub fn unregister_static_collision(&mut self, aabb1: AABB) {
        if let Some(index) = self.static_collisions.iter().position(|&aabb2| aabb1 == aabb2 ) {
            self.static_collisions.swap_remove(index);
        }
    }

    /**
        Store the next values of the pathfinding graph in the path data.

        Returns `true` if the last node was reached.
    */
    pub fn compute_path(&self, path_data: &mut PathFindingData) -> bool {
        let graph = &self.graphs[path_data.graph_id as usize];
        let nodes = graph.nodes.borrow();
        let next_node_index = (path_data.graph_node_index + 1) as usize;

        if next_node_index >= nodes.len() {
            return true
        }

        path_data.next_position = nodes[next_node_index];
        path_data.graph_node_index = next_node_index as u32;

        false
    }

    pub fn clear_path(&mut self, path_data: PathFindingData) {
        let graph = &mut self.graphs[path_data.graph_id as usize];
        graph.nodes.get_mut().clear();
        graph.free = true;
    }

    //
    // Debugging tools
    //

    #[cfg(feature="debug")]
    pub fn debug_static_collisions(&self, debug: &mut crate::debug::DebugState) {
        for &aabb in self.static_collisions.iter() {
            debug.debug_rect(aabb, [255, 0, 0, 255]);
        }
    }

    #[cfg(feature="debug")]
    pub fn debug_path(&self, debug: &mut crate::debug::DebugState, path_data: &PathFindingData) {
        let graph = &self.graphs[path_data.graph_id as usize];
        let nodes = graph.nodes.borrow();
        if nodes.len() < 2 {
            return;
        }

        let colors = [
            [255, 0, 0, 255],
            [0, 255, 0, 255],
            [0, 0, 255, 255],
        ];

        let mut start = nodes[0];
        let mut color_index = 0;
        for &node in &nodes[1..] {
            if color_index > 2 { color_index = 0; }

            debug.debug_line(start, node, colors[color_index]);
            start = node;

            color_index += 1;
        }
    }

}

fn intersection(vector1: Vec2, vector2: Vec2) -> Option<Position<f32>> {
    let x1 = vector1[0].x; let y1 = vector1[0].y;
    let x2 = vector1[1].x; let y2 = vector1[1].y;

    let x3 = vector2[0].x; let y3 = vector2[0].y;
    let x4 = vector2[1].x; let y4 = vector2[1].y;

    let denominator = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
    if denominator == 0.0 {
        return None;
    }

    let ua = ((x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3)) / denominator;
    let x = x1 + ua * (x2 - x1);
    let y = y1 + ua * (y2 - y1);

    if x >= x3 && x <= x4 && y >= y3 && y <= y4 {
        Some(pos(x, y))
    } else {
        None
    }
}

impl Default for PathfindingState {
    fn default() -> Self {
        PathfindingState {
            static_collisions: Vec::with_capacity(32),
            navmesh: (),
            graphs: Vec::with_capacity(32)
        }
    }
}

impl crate::store::SaveAndLoad for PathfindingState {
    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let static_collisions = reader.read_vec();
        let graphs = reader.load_vec();
        PathfindingState {
            static_collisions,
            navmesh: (),
            graphs
        }
    }

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.static_collisions);
        writer.save_slice(&self.graphs);
    }
}

impl crate::store::SaveAndLoad for PathfindingGraph {

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let nodes = reader.read_vec();
        let free = reader.read_u32() == 1;
        PathfindingGraph {
            nodes: RefCell::new(nodes),
            free,
        }
    }

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        let nodes = self.nodes.borrow();
        writer.write_slice(&nodes);
        writer.write_u32(self.free as u32);
    }

}
