use crate::shared::{Position, AABB, pos};
use super::delaunator::{Point, Triangulation};

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

    pub fn find_nearest_point(&self, point: Position<f32>) -> Position<f32> {
        fn step(nav: &NavMesh, last: usize, target: Point) -> usize {
            let point = nav.point_of_edge(last);
            let sibling = nav.sibling_of_edge(last);
            let clockwise = orient_point(point, sibling, target);
            last
        }
        
        let base = 0;
        let mut last = base;
        let mut nearest = base;
        loop {
            nearest = step(self, last, point);
            if nearest == base || nearest == last {
                break;
            }

            last = nearest;
        }

        let point_index = self.triangulation.triangles[nearest];
        let point = self.points[point_index];

        pos(point.x, point.y)
    }

    //
    // Helpers methods
    //

    /// Return the starting point associated with halfedge `e`
    #[inline(always)]
    pub fn point_of_edge(&self, e: usize) -> Position<f32> {
        self.points[self.triangulation.triangles[e]]
    }

    /// Return the halfedge starting at the end of edge `e`
    #[inline(always)]
    pub fn sibling_of_edge(&self, e: usize) -> Position<f32> {
        let e = super::delaunator::next_halfedge(e);
        self.points[self.triangulation.triangles[e]]
    }

    pub fn edges_of_triangle(&self, triangle: usize) -> [usize; 3] {
        [3*triangle+0, 3*triangle+1, 3*triangle+2]
    }

    pub fn triangle_of_edge(&self, edge: usize) -> usize {
        edge / 3
    }

    pub fn triangle(&self, triangle: usize) -> [Position<f32>; 3] {
        let [e0, e1, e2] = self.edges_of_triangle(triangle);
        let tris = &self.triangulation.triangles;
        let p0 = self.points[tris[e0]];
        let p1 = self.points[tris[e1]];
        let p2 = self.points[tris[e2]];
        [p0, p1, p2]
    }

}

/// Returns a **negative** value if `p1`, `p2` and `p3` occur in counterclockwise order
/// Returns a **positive** value if they occur in clockwise order
/// Returns zero is they are collinear
fn orient_point(p1: Position<f32>, p2: Position<f32>, p3: Position<f32>) -> f32 {
    // robust-rs orients Y-axis upwards, our convention is Y downwards. This means that the interpretation of the result must be flipped
    robust::orient2d(p1.into(), p2.into(), p3.into()) as f32
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
