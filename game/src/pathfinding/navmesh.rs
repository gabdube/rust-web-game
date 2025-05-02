use crate::shared::{pos, Position, AABB};
use super::delaunator::{Point, Triangulation};

/// The identifier of a triangle in the navmesh
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct NavTriangle(u32);

impl NavTriangle {
    pub fn outside(&self) -> bool {
        self.0 == u32::MAX
    }
    
    pub fn edge(&self) -> u32 {
        self.0 * 3
    }
}

#[derive(Copy, Clone)]
struct StepState {
    target: Position<f32>,
    edge: usize,
    done: bool,
}

impl StepState {
    fn init(target: Position<f32>, start: u32) -> Self {
        StepState {
            target,
            edge: start as usize,
            done: false,
        }
    }
}

#[derive(Default)]
pub struct NavMesh {
    pub points: Vec<Point>,
    pub triangulation: Triangulation,
}

impl NavMesh {
    
    pub fn generate(&mut self) {
        super::delaunator::triangulate_from(&mut self.triangulation, &self.points);
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.triangulation.triangles.clear();
        self.triangulation.halfedges.clear();
        self.triangulation.hull.clear();
    } 

    pub fn push_point(&mut self, x: f32, y: f32) {
        self.points.push(Point { x, y })
    }

    pub fn push_aabb(&mut self, aabb: AABB) {
        let x1 = aabb.left;
        let y1 = aabb.top;
        let x2 = aabb.right;
        let y2 = aabb.bottom;

        self.points.extend_from_slice(&[
            Point { x: x1, y: y1 },
            Point { x: x1, y: y2 },
            Point { x: x2, y: y1 },
            Point { x: x2, y: y2 },
        ]);
    }

    // `start`` and `end` are in the same triangle
    fn same_triangle_path(&self, start_triangle: NavTriangle, end: Position<f32>) -> bool {
        let [p1, p2, p3] = self.triangle_points(start_triangle);
        if inside_triangle(end, p1, p2, p3) {
            true
        } else {
            false
        }
    }

    // If end is in a neighboring triangle from `start`
    // Triangle are considered neighboring if they share a border (aka two points)
    fn neighbors_path(&self, start_triangle: NavTriangle, end_triangle: NavTriangle) -> bool {
        let points1 = self.triangle_points(start_triangle);
        let points2 = self.triangle_points(end_triangle);

        let mut same_points = 0;
        for p1 in points1 {
            same_points += (p1 == points2[0]) as u32;
            same_points += (p1 == points2[1]) as u32;
            same_points += (p1 == points2[2]) as u32;
        }

        same_points > 1
    }

    // Returns `true` if a path was found from `start` to `end`, or false if the target is blocked or out of the navmesh
    pub fn build_path(&self, start: Position<f32>, end: Position<f32>, nodes: &mut Vec<Position<f32>>) -> bool  {
        let start_triangle = self.find_triangle(start, 0);

        // If end is outside the navmesh, we just cancel the pathfinding
        if start_triangle.outside() {
            nodes.clear();
            return false
        }

        if self.same_triangle_path(start_triangle, end) {
            nodes.push(start);
            nodes.push(end);
            return true;
        }

        let start_edge = self.triangle_edges(start_triangle)[0] as u32;
        let end_triangle = self.find_triangle(end, start_edge);

        if self.neighbors_path(start_triangle, end_triangle) {
            nodes.push(start);
            nodes.push(end);
            return true;
        }

        super::navmesh_astar::find_path(self, nodes, start_triangle, end_triangle)
    }

    /// Find the nearest point on the hull from `point`
    pub fn find_nearest_hull_point(&self, point: Position<f32>) -> Position<f32> {
        let mut distance = f32::INFINITY;
        let mut nearest_point = pos(0.0, 0.0);

        for &index in self.triangulation.hull.iter() {
            let point2 = self.points[index];
            let distance2 = point.distance(point2);
            if distance2 < distance {
                distance = distance2;
                nearest_point = point2;
            }
        }

        nearest_point
    }

    /// Find the triangle containing `point` and return the nearest point
    /// If the point is outside the mesh, return the nearest point on the hull
    pub fn find_nearest_point(&self, point: Position<f32>, start_edge: u32) -> Position<f32> {
        let mut state = StepState::init(point, start_edge);
        loop {
            self.step(&mut state);
            if state.done || state.edge == usize::MAX {
                break;
            }
        }

        // Point is outside the hull
        if state.edge == usize::MAX {
            return self.find_nearest_hull_point(point);   
        }

        let triangle = self.triangle_of_edge(state.edge);
        let [e1, e2, e3] = self.triangle_edges(triangle);
        let [p1, p2, p3] = self.triangle_points(triangle);

        let d1 = point.distance(p1);
        let d2 = point.distance(p2);
        let d3 = point.distance(p3);

        let min;
        if d1 < d2 && d1 < d3 {
            min = e1;
        } else if d2 < d1 && d2 < d3 {
            min = e2;
        } else {
            min = e3;
        }

        self.point_of_edge(min)
    }

    /// Find the triangle containing `point` in the navmesh. 
    /// If the point is outside the nav mesh, `triangle.outside` will return true
    pub fn find_triangle(&self, point: Position<f32>, start_edge: u32) -> NavTriangle {
        let mut state = StepState::init(point, start_edge);
        loop {
            self.step(&mut state);
            if state.done || state.edge == usize::MAX {
                break;
            }
        }

        self.triangle_of_edge(state.edge)
    }

    pub fn iter_path<CB>(&self, start_point: Position<f32>, stop_point: Position<f32>, start_edge: u32, mut on_step: CB) 
        where CB: FnMut(NavTriangle)
    {
        let start_triangle = self.find_triangle(start_point, start_edge);
        if start_triangle.outside() {
            return;
        }

        on_step(start_triangle);

        let edge = 3 * start_triangle.0;
        let mut state = StepState::init(stop_point, edge);
        loop {
            self.step(&mut state);
            if state.done || state.edge == usize::MAX {
                break;
            }

            on_step(self.triangle_of_edge(state.edge));
        }
    }

    //
    // Helpers methods
    //

    fn step(&self, step: &mut StepState) {
        let sibling = super::delaunator::next_halfedge(step.edge);
        let last_sibling = super::delaunator::next_halfedge(sibling);
        let point1 = self.point_of_edge(step.edge);
        let point2 = self.point_of_edge(sibling);
        let point3 = self.point_of_edge(last_sibling);

        // If `target` is not counterclockwise (cc), this means the point is in another triangle.
        // If so we test the opposite half edge in the next iteration
        let mut next_edge = None;
        if orient_point(point1, point2, step.target) > 0.0 {
            next_edge = Some(self.triangulation.halfedges[step.edge]);
        }
        else if orient_point(point2, point3, step.target) > 0.0 {
            next_edge = Some(self.triangulation.halfedges[sibling]);
        }
        else if orient_point(point3, point1, step.target) > 0.0 {
            next_edge = Some(self.triangulation.halfedges[last_sibling]);
        }

        if let Some(next_edge) = next_edge {
            step.edge = next_edge;
            return;
        }

        step.done = true;
    }

    pub fn triangle_edges(&self, triangle: NavTriangle) -> [usize; 3] {
        let triangle_index = triangle.0 as usize;
        if triangle_index == (u32::MAX as usize) {
            return [Default::default(); 3];
        }
        
        [ 3*triangle_index+0, 3*triangle_index+1, 3*triangle_index+2 ]
    }

    pub fn triangle_points(&self, triangle: NavTriangle) -> [Position<f32>; 3] {
        let triangle_index = triangle.0 as usize;
        if triangle_index == (u32::MAX as usize) {
            return [Default::default(); 3];
        }

        [
            self.point_of_edge(3*triangle_index+0),
            self.point_of_edge(3*triangle_index+1),
            self.point_of_edge(3*triangle_index+2),
        ]
    }

    pub fn neighbors_of_triangle(&self, triangle: NavTriangle) -> [NavTriangle; 3] {
        let edges = self.triangle_edges(triangle);
        let halfedges = &self.triangulation.halfedges;

        let mut h1 = halfedges[edges[0]] as u32;
        let mut h2 = halfedges[edges[1]] as u32;
        let mut h3 = halfedges[edges[2]] as u32;

        if h1 != u32::MAX {  h1 = h1 / 3; }
        if h2 != u32::MAX {  h2 = h2 / 3; }
        if h3 != u32::MAX {  h3 = h3 / 3; }

        [
            NavTriangle(h1),
            NavTriangle(h2),
            NavTriangle(h3),
        ]
    }

    /// Return the starting point associated with halfedge `e`
    #[inline(always)]
    pub fn point_of_edge(&self, e: usize) -> Position<f32> {
        self.points[self.triangulation.triangles[e]]
    }

    #[inline(always)]
    pub fn triangle_of_edge(&self, edge: usize) -> NavTriangle {
        if edge == usize::MAX {
            NavTriangle(u32::MAX)
        } else {
            NavTriangle((edge / 3) as u32)
        }
    }

}

/// Returns a **negative** value if `p1`, `p2` and `p3` occur in counterclockwise order
/// Returns a **positive** value if they occur in clockwise order
/// Returns zero is they are collinear
fn orient_point(p1: Position<f32>, p2: Position<f32>, p3: Position<f32>) -> f32 {
    // robust-rs orients Y-axis upwards, our convention is Y downwards. This means that the interpretation of the result must be flipped
    robust::orient2d(p1.into(), p2.into(), p3.into()) as f32
}

fn inside_triangle(point: Position<f32>, p1: Position<f32>, p2: Position<f32>, p3: Position<f32>) -> bool {
    orient_point(p1, p2, point) < 0.0 && orient_point(p2, p3, point) < 0.0 && orient_point(p3, p1, point) < 0.0
}


fn intersection(p1: Position<f32>, p2: Position<f32>, p3: Position<f32>, p4: Position<f32>) -> Option<Position<f32>> {
    let x1 = p1.x; let y1 = p1.y;
    let x2 = p2.x; let y2 = p2.y;
    let x3 = p3.x; let y3 = p3.y;
    let x4 = p4.x; let y4 = p4.y;

    let denominator = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
    if denominator == 0.0 {
        return None;
    }

    let ua = ((x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3)) / denominator;
    let x = x1 + ua * (x2 - x1);
    let y = y1 + ua * (y2 - y1);

    if x < f32::max(f32::min(x1, x2), f32::min(x3, x4)) || x > f32::min(f32::max(x1, x2), f32::max(x3, x4)) {
        return None;
    }

    Some(pos(x, y))
}

impl crate::store::SaveAndLoad for NavMesh {

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let points = reader.read_vec();
        let triangles = reader.read_vec();
        let halfedges = reader.read_vec();
        let hull = reader.read_vec();
        NavMesh { 
            points,
            triangulation: Triangulation { triangles, halfedges, hull }
        }
    }

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.points);
        writer.write_slice(&self.triangulation.triangles);
        writer.write_slice(&self.triangulation.halfedges);
        writer.write_slice(&self.triangulation.hull);
    }
}
