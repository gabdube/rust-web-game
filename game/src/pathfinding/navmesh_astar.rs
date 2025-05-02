//! AStar pathfinding over a `NavMesh`
use std::collections::BinaryHeap;
use crate::shared::Position;
use super::navmesh::{NavMesh, NavTriangle};

struct NavCell {
    triangle: NavTriangle,
}

pub(super) fn find_path<'a>(
    nav: &'a NavMesh,
    nodes: &'a mut Vec<Position<f32>>,
    start_triangle: NavTriangle,
    end_triangle: NavTriangle,
) -> bool {

    // An array of triangle to traverse to reach `end_triangle` from `start_triangle`
    let mut triangle_strip = Vec::with_capacity(8);
    triangle_strip.push(start_triangle);
    find_triangle_strip(nav, &mut triangle_strip);

    false
}

fn find_triangle_strip<'a>(nav: &'a NavMesh, triangles: &mut Vec<NavTriangle>) {

}
