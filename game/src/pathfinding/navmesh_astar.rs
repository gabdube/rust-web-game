//! AStar pathfinding over a `NavMesh`
use fnv::FnvHashMap;
use std::collections::BinaryHeap;
use std::cmp::Ord;
use crate::shared::Position;
use super::navmesh::{NavMesh, NavTriangle};

struct NavCell {
    triangle: NavTriangle,
    circumcenter: Position<f32>,
    cost: f32,
}

pub(super) fn find_path<'a>(
    nav: &'a NavMesh,
    nodes: &'a mut Vec<Position<f32>>,
    start_triangle: NavTriangle,
    end_triangle: NavTriangle,
) -> bool {

    // Generate an array of triangle to traverse to reach `end_triangle` from `start_triangle`
    let mut triangle_strip = Vec::with_capacity(8);
    find_triangle_strip(nav, start_triangle, end_triangle, &mut triangle_strip);


    false
}


fn reverse_path(parents: &FnvHashMap<NavTriangle, (NavTriangle, f32)>, triangles: &mut Vec<NavTriangle>, end: NavTriangle) {
    let mut current = end;
    let (mut parent, _) = parents.get(&end).unwrap();
    while current != parent {
        triangles.push(parent);
        current = parent;
        parent = parents.get(&current).unwrap().0;
    }

    triangles.reverse();
}

/// This is the actual a* algorithm
pub(super) fn find_triangle_strip<'a>(
    nav: &'a NavMesh,
    start_triangle: NavTriangle,
    end_triangle: NavTriangle,
    triangles: &mut Vec<NavTriangle>
) -> bool {
    use std::collections::hash_map::Entry;

    let center_start = {
        let [p1, p2, p3] = nav.triangle_points(start_triangle);
        circumcenter(p1, p2, p3)
    };

    let mut to_see = BinaryHeap::new();
    to_see.push(NavCell { 
        triangle: start_triangle,
        circumcenter: center_start,
        cost: 0.0,
    });

    let mut parents = FnvHashMap::default();
    parents.insert(start_triangle, (start_triangle, 0.0));

    while let Some(cell) = to_see.pop() {
        if cell.triangle == end_triangle {
            triangles.push(cell.triangle);
            reverse_path(&parents, triangles, end_triangle);
            return true;
        }

        let neighbors = nav.neighbors_of_triangle(cell.triangle);
        for &neighbor in neighbors.iter() {
            if neighbor.outside() {
                continue;
            }

            // The cost is the distance from the circumcenter of the neighbor triangle to the circumcenter of current triangle
            let [p1, p2, p3] = nav.triangle_points(neighbor);
            let center = circumcenter(p1, p2, p3);
            let distance = cell.circumcenter.distance(center);
            let cost = cell.cost + distance;

            match parents.entry(neighbor) {
                Entry::Vacant(p) => {
                    p.insert((cell.triangle, cost));
                }
                Entry::Occupied(mut p) => {
                    let old_cost = p.get().1;
                    if old_cost > cost {
                        p.insert((cell.triangle, cost));
                    } else {
                        continue;
                    }
                }
            }

            to_see.push(NavCell { 
                triangle: neighbor,
                circumcenter: center,
                cost
            });
        }
    }

    return false;
}

//
// Helpers
//

fn circumdelta(p1: Position<f32>, p2: Position<f32>, p3: Position<f32>) -> (f32, f32) {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let ex = p3.x - p1.x;
    let ey = p3.y - p1.y;

    let bl = dx * dx + dy * dy;
    let cl = ex * ex + ey * ey;
    let d = 0.5 / (dx * ey - dy * ex);

    let x = (ey * bl - dy * cl) * d;
    let y = (dx * cl - ex * bl) * d;
    (x, y)
}

fn circumcenter(p1: Position<f32>, p2: Position<f32>, p3: Position<f32>) -> Position<f32> {
    let (x, y) = circumdelta(p1, p2, p3);
    Position {
        x: p1.x + x,
        y: p1.y + y,
    }
}

//
// Other Impl
//


impl Ord for NavCell {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.total_cmp(&self.cost)
    }
}

impl PartialOrd for NavCell {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for NavCell {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for NavCell {}

