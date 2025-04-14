mod delaunator;
use delaunator::{Triangulation, Point};

use crate::shared::{pos, Position, AABB};

/// Computed pathfinding data for a single unit 
#[derive(Copy, Clone)]
pub struct PathFindingData {
    pub next_position: Position<f32>,
    pub graph_id: u32,
    pub graph_node_index: u32,
}

/**
    A node graph of pathfinding points a character has to traverse to reach a destination
*/
pub struct PathfindingGraph {
    pub nodes: Vec<Position<f32>>,
    pub free: bool,
}

pub struct NavMesh {
    pub terrain_bounds: Vec<Point>,
    pub static_aabbs: Vec<AABB>,
    pub points: Vec<Point>,
    pub triangulation: Triangulation,
}

/// Global pathfinding state
pub struct PathfindingState {
    pub navmesh: NavMesh,
    pub paths: Vec<PathfindingGraph>
}

impl PathfindingState {

    pub fn generate_navmesh(&mut self) {
        let points = &mut self.navmesh.points;
    
        points.clear();

        // Add the world bounds
        for &point in self.navmesh.terrain_bounds.iter() {
            points.push(point);
        }

        // Add the static objects
        for &aabb in self.navmesh.static_aabbs.iter() {
            points.push(Point::from([aabb.left, aabb.top]));
            points.push(Point::from([aabb.left, aabb.bottom]));
            points.push(Point::from([aabb.right, aabb.top]));
            points.push(Point::from([aabb.right, aabb.bottom]));
        }

        delaunator::triangulate_from(&mut self.navmesh.triangulation, &points);
    }
   
    pub fn clear(&mut self) {
        self.navmesh.clear();

        for graph in self.paths.iter_mut() {
            graph.nodes.clear();
            graph.free = true;
        }
    }

    //
    // Navmesh generation
    //

    pub fn clear_terrain_bounds(&mut self) {
        self.navmesh.terrain_bounds.clear();
    }

    pub fn add_terrain_bound(&mut self, x: f32, y: f32) {
        self.navmesh.terrain_bounds.push(Point::from([x, y]));
    }

    pub fn register_static_collision(&mut self, aabb: AABB) {
        self.navmesh.static_aabbs.push(aabb);
    }

    pub fn unregister_static_collision(&mut self, aabb1: AABB) {
        if let Some(index) = self.navmesh.static_aabbs.iter().position(|&aabb2| aabb1 == aabb2 ) {
            self.navmesh.static_aabbs.swap_remove(index);
        }
    }

    //
    // Pathing
    //

    /**
        Compute the path from `start_position` to `final_position`. Once used,
        the returned `PathFindingData` must be freed using `free_path`.
    */
    pub fn compute_new_path(&mut self, start_position: Position<f32>, final_position: Position<f32>) -> PathFindingData {
        let graph_id = self.find_new_path();
        self.paths[graph_id as usize].nodes.push(start_position);
        self.paths[graph_id as usize].nodes.push(final_position);

        PathFindingData {
            next_position: start_position,
            graph_id,
            graph_node_index: 0,
        }
    }

    /**
        Store the next values of the pathfinding graph in the path data.

        Returns `true` if the last node was reached.
    */
    pub fn compute_path(&self, path_data: &mut PathFindingData) -> bool {
        let graph = &self.paths[path_data.graph_id as usize];
        let next_node_index = (path_data.graph_node_index + 1) as usize;

        if next_node_index >= graph.nodes.len() {
            return true
        }

        path_data.next_position = graph.nodes[next_node_index];
        path_data.graph_node_index = next_node_index as u32;

        false
    }

    /**
        Free up `path_data`. Allocated memory will be reused by by the next call
        to `compute_new_path`
    */
    pub fn free_path(&mut self, path_data: PathFindingData) {
        let graph = &mut self.paths[path_data.graph_id as usize];
        graph.nodes.clear();
        graph.free = true;
    }
 
    fn find_new_path(&mut self) -> u32 {
        let graph_id;
        match self.paths.iter_mut().enumerate().find(|(_,g)| g.free ) {
            Some((index, graph)) => {
                graph_id = index as u32;
                graph.free = false;
            },
            None => {
                graph_id = self.paths.len() as u32;
                self.paths.push(PathfindingGraph {
                    nodes: Vec::with_capacity(8),
                    free: false,
                });
            }
        }

        graph_id
    }

    //
    // Debugging tools
    //

    #[cfg(feature="debug")]
    #[allow(dead_code)]
    pub fn debug_static_collisions(&self, debug: &mut crate::debug::DebugState) {
        let color = [0, 255, 0, 255];
        for &aabb in self.navmesh.static_aabbs.iter() {
            debug.debug_rect(aabb, color);
        }
    }

    #[cfg(feature="debug")]
    #[allow(dead_code)]
    pub fn debug_navmesh(&self, debug: &mut crate::debug::DebugState) {
        let points = &self.navmesh.points;
        let triangles = &self.navmesh.triangulation.triangles;
        let triangle_count = triangles.len();
        let color = [255, 0, 0, 255];

        let mut i = 0;
        while i < triangle_count {
            let p1 = points[triangles[i]];
            let p2 = points[triangles[i+1]];
            let p3 = points[triangles[i+2]];
            debug.debug_triangle(
                pos(p1.x, p1.y),
                pos(p2.x, p2.y),
                pos(p3.x, p3.y),
                color
            );
            i += 3;
        }
    }

    #[cfg(feature="debug")]
    #[allow(dead_code)]
    pub fn debug_path(&self, debug: &mut crate::debug::DebugState, path_data: &PathFindingData) {
        let graph = &self.paths[path_data.graph_id as usize];
        if graph.nodes.len() < 2 {
            return;
        }

        let colors = [
            [255, 0, 0, 255],
            [0, 255, 0, 255],
            [0, 0, 255, 255],
        ];

        let mut start = graph.nodes[0];
        let mut color_index = 0;
        for &node in &graph.nodes[1..] {
            if color_index > 2 { color_index = 0; }

            debug.debug_line(start, node, colors[color_index]);
            start = node;

            color_index += 1;
        }
    }

    pub fn debug_pathfinding(&self, debug: &mut crate::debug::DebugState, start: Position<f32>, end: Position<f32>) {

    }

}

impl NavMesh {
    fn clear(&mut self) {
        self.terrain_bounds.clear();
        self.static_aabbs.clear();
        self.points.clear();
        self.triangulation.triangles.clear();
        self.triangulation.halfedges.clear();
        self.triangulation.hull.clear();
    } 
}

impl Default for NavMesh {
    fn default() -> Self {
        NavMesh {
            terrain_bounds: Vec::with_capacity(64),
            static_aabbs: Vec::with_capacity(64),
            points: Vec::with_capacity(400),
            triangulation: Triangulation {
                triangles: Vec::new(),
                halfedges: Vec::new(),
                hull: Vec::new(),
            }
        }
    }
}

impl Default for PathfindingState {
    fn default() -> Self {
        PathfindingState {
            navmesh: NavMesh::default(),
            paths: Vec::with_capacity(32)
        }
    }
}

impl crate::store::SaveAndLoad for PathfindingState {
    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let navmesh = reader.load();
        let paths = reader.load_vec();
        PathfindingState {
            navmesh,
            paths
        }
    }

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.save(&self.navmesh);
        writer.save_slice(&self.paths);
    }
}

impl crate::store::SaveAndLoad for NavMesh {

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let terrain_bounds = reader.read_vec();
        let static_aabbs = reader.read_vec();
        let points = reader.read_vec();
        let triangles = reader.read_vec();
        let halfedges = reader.read_vec();
        let hull = reader.read_vec();
        NavMesh { 
            terrain_bounds,
            static_aabbs,
            points,
            triangulation: Triangulation { triangles, halfedges, hull }
        }
    }

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.terrain_bounds);
        writer.write_slice(&self.static_aabbs);
        writer.write_slice(&self.points);
        writer.write_slice(&self.triangulation.triangles);
        writer.write_slice(&self.triangulation.halfedges);
        writer.write_slice(&self.triangulation.hull);
    }
}

impl crate::store::SaveAndLoad for PathfindingGraph {

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let nodes = reader.read_vec();
        let free = reader.read_u32() == 1;
        PathfindingGraph {
            nodes,
            free,
        }
    }

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.nodes);
        writer.write_u32(self.free as u32);
    }

}
