use crate::shared::{pos, Position, AABB};
use super::delaunator::{Point, Triangulation};

/// The identifier of a triangle in the navmesh
struct Triangle(pub u32);

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

    pub fn find_nearest_point(&self, point: Position<f32>) -> Position<f32> {
        struct StepState {
            target: Position<f32>,
            edge: usize,
            done: bool,
        }

        fn init_step(point: Position<f32>) -> StepState {
            let start = 0; // First halfedge in the triangulation
            StepState {
                target: point,
                edge: start,
                done: false,
            }
        }
        
        fn step(nav: &NavMesh, step: &mut StepState) {
            let sibling = super::delaunator::next_halfedge(step.edge);
            let last_sibling = super::delaunator::next_halfedge(sibling);
            let point1 = nav.point_of_edge(step.edge);
            let point2 = nav.point_of_edge(sibling);
            let point3 = nav.point_of_edge(last_sibling);

            // If `target` is not counterclockwise (cc), this means the point is in another triangle.
            // If so we test the opposite half edge in the next iteration
            let mut next_edge = None;
            if orient_point(point1, point2, step.target) > 0.0 {
                next_edge = Some(nav.triangulation.halfedges[step.edge]);
            }
            else if orient_point(point2, point3, step.target) > 0.0 {
                next_edge = Some(nav.triangulation.halfedges[sibling]);
            }
            else if orient_point(point3, point1, step.target) > 0.0 {
                next_edge = Some(nav.triangulation.halfedges[last_sibling]);
            }

            if let Some(next_edge) = next_edge {
                step.edge = next_edge;
                return;
            }

            // If the point is counterclockwise to all halfedge in the current triangle
            // this means the point is inside the current triangle, so we pick 
            // the closest point to the target
            let d1 = step.target.distance(point1);
            let d2 = step.target.distance(point2);
            let d3 = step.target.distance(point3);

            if d1 < d2 && d1 < d3 {
                step.edge = step.edge
            } else if d2 < d1 && d2 < d3 {
                step.edge = sibling
            } else {
                step.edge = last_sibling
            }

            step.done = true;
        }

        let mut state = init_step(point);
        loop {
            step(self, &mut state);
            if state.done || state.edge == usize::MAX {
                break;
            }
        }

        if state.edge == usize::MAX {
            self.find_nearest_hull_point(point)   // Point is outside the hull
        } else {
            self.point_of_edge(state.edge)
        }
    }


    //
    // Helpers methods
    //

    /// Return the starting point associated with halfedge `e`
    #[inline(always)]
    pub fn point_of_edge(&self, e: usize) -> Position<f32> {
        self.points[self.triangulation.triangles[e]]
    }

    // pub fn edges_of_triangle(&self, triangle: usize) -> [usize; 3] {
    //     [3*triangle+0, 3*triangle+1, 3*triangle+2]
    // }

    // pub fn triangle_of_edge(&self, edge: usize) -> Triangle {
    //     Triangle((edge / 3) as u32)
    // }

    // pub fn triangle_points(&self, triangle: usize) -> [Position<f32>; 3] {
    //     let [e0, e1, e2] = self.edges_of_triangle(triangle);
    //     let tris = &self.triangulation.triangles;
    //     let p0 = self.points[tris[e0]];
    //     let p1 = self.points[tris[e1]];
    //     let p2 = self.points[tris[e2]];
    //     [p0, p1, p2]
    // }

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
