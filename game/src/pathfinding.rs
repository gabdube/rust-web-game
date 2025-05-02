mod delaunator;

mod navmesh;
use navmesh::NavMesh;

mod navmesh_astar;

use crate::shared::Position;

/// Computed pathfinding data for a single unit 
#[derive(Copy, Clone)]
pub struct PathFindingData {
    pub next_position: Position<f32>,
    pub path_id: u32,
    pub current_node_index: u32,
}

/**
    A node graph of pathfinding points a character has to traverse to reach a destination
*/
pub struct PathfindingGraph {
    pub nodes: Vec<Position<f32>>,
    pub free: bool,
}

/// Global pathfinding state
pub struct PathfindingState {
    pub navmesh: NavMesh,
    pub paths: Vec<PathfindingGraph>
}

impl PathfindingState {

    pub fn clear(&mut self) {
        self.navmesh.clear();

        for graph in self.paths.iter_mut() {
            graph.nodes.clear();
            graph.free = true;
        }
    }


    //
    // Pathing
    //

    /**
        Compute the path from `start` to `end`. Once used,
        the returned `PathFindingData` must be freed using `free_path`.
    */
    pub fn compute_new_path(&mut self, start: Position<f32>, end: Position<f32>) -> Option<PathFindingData> {
        let path_id = self.new_path();
        let path = &mut self.paths[path_id as usize];
        let nodes = &mut path.nodes;
        let valid = self.navmesh.build_path(start, end, nodes);
        if !valid {
            path.free = true;
            return None;
        }

        let path_data = PathFindingData {
            next_position: start,
            path_id,
            current_node_index: 0,
        };

        Some(path_data)
    }

    /**
        Store the next values of the pathfinding graph in the path data.

        Returns `true` if the last node was reached.
    */
    pub fn compute_path(&self, path_data: &mut PathFindingData) -> bool {
        let graph = &self.paths[path_data.path_id as usize];
        let next_node_index = (path_data.current_node_index + 1) as usize;

        if next_node_index >= graph.nodes.len() {
            return true
        }

        path_data.next_position = graph.nodes[next_node_index];
        path_data.current_node_index = next_node_index as u32;

        false
    }

    /**
        Free up `path_data`. Allocated memory will be reused by by the next call
        to `compute_new_path`
    */
    pub fn free_path(&mut self, path_data: PathFindingData) {
        let graph = &mut self.paths[path_data.path_id as usize];
        graph.nodes.clear();
        graph.free = true;
    }
 
    fn new_path(&mut self) -> u32 {
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
    pub fn debug_navmesh(&self, debug: &mut crate::debug::DebugState) {
        let triangles = &self.navmesh.triangulation.triangles;
        let triangle_count = triangles.len();
        let color = [255, 0, 0, 255];

        let mut i = 0;
        while i < triangle_count {
            let triangle = self.navmesh.triangle_of_edge(i);
            let [p1, p2, p3] = self.navmesh.triangle_points(triangle); 
            debug.debug_triangle(p1, p2, p3, color);
            i += 3;
        }
    }

    #[cfg(feature="debug")]
    #[allow(dead_code)]
    pub fn debug_path(&self, debug: &mut crate::debug::DebugState, path_data: &PathFindingData) {
        let graph = &self.paths[path_data.path_id as usize];
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

    #[cfg(feature="debug")]
    #[allow(dead_code)]
    pub fn debug_pathfinding(
        &self,
        debug: &mut crate::debug::DebugState,
        start: Position<f32>,
        end: Position<f32>
    ) {
        let green = [0, 255, 0, 255];
        let blue = [0, 0, 255, 255];
        let mut nodes = Vec::with_capacity(8);

        // let start_triangle = self.navmesh.find_triangle(start, 0);
        // let [p0, p1, p2] = self.navmesh.triangle_points(start_triangle);
        // debug.debug_triangle_fill(p0, p1, p2, [255,255,255, 50]);

        self.navmesh.build_path(start, end, &mut nodes);

        if nodes.len() >= 2 {
            let mut last = nodes.first().copied().unwrap();
            for i in 1..nodes.len() {
                debug.debug_line(last, nodes[i], green);
                debug.debug_point(nodes[i], 10.0, blue);
                last = nodes[i];
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
