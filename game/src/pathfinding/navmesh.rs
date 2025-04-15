use crate::shared::AABB;
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

