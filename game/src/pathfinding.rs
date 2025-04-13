/*! 
    2D non tile-based pathfinding
*/ 
use std::cell::RefCell;
use crate::shared::{pos, Position, AABB};

type Vec2 = [Position<f32>; 2];

/// Computed pathfinding data for a single unit 
#[derive(Copy, Clone)]
pub struct PathFindingData {
    pub next_position: Position<f32>,
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
    pub free: bool,
}

/// Global pathfinding state
pub struct PathfindingState {
    pub blocked: Vec<AABB>,
    pub graphs: Vec<PathfindingGraph>
}

impl PathfindingState {

    pub fn new(&mut self, start_position: Position<f32>, final_position: Position<f32>) -> PathFindingData {
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

        self.build_graph(start_position, final_position, graph_id);

        PathFindingData {
            next_position: start_position,
            graph_id,
            graph_node_index: 0,
        }
    }

    pub fn clear(&mut self) {
        self.blocked.clear();

        for graph in self.graphs.iter_mut() {
            graph.nodes.get_mut().clear();
            graph.free = true;
        }
    }

    pub fn register_collision(&mut self, aabb: AABB) {
        self.blocked.push(aabb);
    }

    pub fn unregister_collision(&mut self, aabb1: AABB) {
        if let Some(index) = self.blocked.iter().position(|&aabb2| aabb1 == aabb2 ) {
            self.blocked.swap_remove(index);
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

    /// Find the nearest collision between `vector1` and the bounding boxes registered in the pathfinding utility
    fn collision(&self, vector1: Vec2) -> Option<Position<f32>> {
        const fn vec(v1: f32, v2: f32, v3: f32, v4: f32) -> Vec2 {
            [pos(v1, v2), pos(v3, v4)]
        }

        fn min_distance(
            v1: Vec2,
            v2: Vec2,
            nearest_distance: &mut f32,
            nearest_intersection: &mut Option<Position<f32>>
        ) {
            if let Some(point) = intersection(v1, v2) {
                let distance = point.distance(v1[0]);
                if distance < *nearest_distance {
                    *nearest_distance = distance;
                    *nearest_intersection = Some(point);
                }
            }
        }
        
        let x1 = vector1[0].x;
        let y1 = vector1[0].y;
        let x2 = vector1[1].x;
        let y2 = vector1[1].y;
        let base = AABB {
            left: f32::min(x1, x2),
            right: f32::max(x1, x2),
            top: f32::min(y1, y2),
            bottom: f32::max(y1, y2),
        };

        let mut nearest_intersection = None;
        let mut nearest_distance = f32::INFINITY;
        let mut vector2;

        for aabb in self.blocked.iter() {
            if !base.intersects(aabb) {
                continue;
            }

            // Top
            vector2 = vec(aabb.left, aabb.top, aabb.right, aabb.top);
            min_distance(vector1, vector2, &mut nearest_distance, &mut nearest_intersection);

            // Bottom
            vector2 = vec(aabb.left, aabb.bottom, aabb.right, aabb.bottom);
            min_distance(vector1, vector2, &mut nearest_distance, &mut nearest_intersection);

            // Left
            vector2 = vec(aabb.left, aabb.top, aabb.left, aabb.bottom);
            min_distance(vector1, vector2, &mut nearest_distance, &mut nearest_intersection);

            // Right
            vector2 = vec(aabb.right, aabb.top, aabb.right, aabb.bottom);
            min_distance(vector1, vector2, &mut nearest_distance, &mut nearest_intersection);
        } 

        nearest_intersection
    }

    /**
        Finds a path that go from `current_position` to `final_position`.
    */
    fn build_graph(&self, start: Position<f32>, end: Position<f32>, graph_index: u32) {
        let graph = &self.graphs[graph_index as usize];
        let mut nodes = graph.nodes.borrow_mut();
        
        nodes.clear();
        nodes.push(start);

        if let Some(intersection) = self.collision([start, end]) {
            nodes.push(intersection);
        }

        nodes.push(end);
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
            blocked: Vec::with_capacity(32),
            graphs: Vec::with_capacity(32)
        }
    }
}

impl crate::store::SaveAndLoad for PathfindingState {
    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let blocked = reader.read_vec();
        let graphs = reader.load_vec();
        PathfindingState {
            blocked,
            graphs
        }
    }

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.blocked);
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
