//! AStar pathfinding over a `NavMesh`
use fnv::FnvHashMap;
use std::collections::BinaryHeap;
use std::cmp::Ord;
use crate::shared::Position;
use super::navmesh::{NavMesh, NavTriangle};

struct NavCell {
    triangle: NavTriangle,
    point: Position<f32>,
    cost: f32,
}

pub(super) fn find_path<'a>(
    nav: &'a NavMesh,
    nodes: &'a mut Vec<Position<f32>>,
    start_triangle: NavTriangle,
    end_triangle: NavTriangle,
    start: Position<f32>,
    end: Position<f32>,
) -> bool {
    nodes.push(start);
    nodes.push(end);

    true
}

struct NavCell2 {
    triangle: NavTriangle,
    edge: u32,
    position: Position<f32>,
    cost: f32,
}

impl Ord for NavCell2 {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.total_cmp(&self.cost)
    }
}

impl PartialOrd for NavCell2 {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for NavCell2 {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for NavCell2 {}

fn reverse_path2(
    parents: &FnvHashMap<NavTriangle, (NavTriangle, u32, f32)>,
    edges: &mut Vec<u32>,
    start: NavTriangle,
    start_edge: u32,
) {
    edges.push(start_edge);

    let mut count = 0;

    let mut last = start;
    loop {
        let (triangle, edge, _) = match parents.get(&last).copied() {
            Some(value) => value,
            None => { break; }
        };

        if edge == u32::MAX {
            break;
        }

        if count > 10 {
            dbg!("ERROR");
            break;
        }

        edges.push(edge);

        last = triangle;
        count += 1;
    }

    edges.reverse();
}

pub(super) fn debug_path(
    debug: &mut crate::debug::DebugState,
    edges: &mut Vec<u32>,
    nav: &NavMesh,
    start_triangle: NavTriangle,
    end_triangle: NavTriangle,
    start: Position<f32>,
) -> bool  {
    use std::collections::hash_map::Entry;

    let mut to_see: BinaryHeap<NavCell2> = BinaryHeap::new();
    let mut parents = FnvHashMap::default();

    to_see.push(NavCell2 { triangle: start_triangle, edge: u32::MAX, position: start, cost: 0.0 });
    parents.insert(start_triangle, (start_triangle, u32::MAX, 0.0)); // triangle: (parent triangle, connecting_edge, min_distance)

    while let Some(cell) = to_see.pop() {
        if cell.triangle == end_triangle {
            reverse_path2(&parents, edges, cell.triangle, cell.edge);
            return true;
        }

        let edges = sort_edges(nav, cell.triangle, cell.position);
        for (distance, position, edge) in edges {
            let neighbor_edge = nav.triangulation.halfedges[edge as usize] as u32;
            if neighbor_edge != u32::MAX && edge != cell.edge {
                let neigbours_triangle = nav.triangle_of_edge(neighbor_edge as usize);
                let cost = cell.cost + distance;
                match parents.entry(neigbours_triangle) {
                    Entry::Vacant(e) => {
                        e.insert((cell.triangle, neighbor_edge, cost));
                    }
                    Entry::Occupied(mut e) => {
                        if e.get().2 > cost {
                            e.insert((cell.triangle, neighbor_edge, cost));
                        } else {
                            continue;
                        }
                    }
                }

                to_see.push(NavCell2 { triangle: neigbours_triangle, edge: neighbor_edge, position, cost });
            }
        }
    }

    false
}

//
// Helpers
//

/// Sort edges of `triangle` by their distance from `position`. /// Returns an array of `(distance, point, edge)`
/// The distance is computed from either endings of the edge. Because each edge share a point, distance and point at index `0` and `1` will always be the same.
fn sort_edges(nav: &NavMesh, triangle: NavTriangle, position: Position<f32>) -> [(f32, Position<f32>, u32); 3] {
    #[derive(Copy, Clone)]
    struct EdgePoint {
        distance: f32,
        point: Position<f32>,
        edge: usize,
    }
    
    let triangle_index = triangle.index();
    let [e1, e2, e3] = [3*triangle_index+0, 3*triangle_index+1, 3*triangle_index+2];
    let [p1, p2, p3] = [nav.point_of_edge(e1), nav.point_of_edge(e2), nav.point_of_edge(e3)];
    let [d1, d2, d3] = [
        f32::abs(position.x - p1.x) + f32::abs(position.y - p1.y),
        f32::abs(position.x - p2.x) + f32::abs(position.y - p2.y),
        f32::abs(position.x - p3.x) + f32::abs(position.y - p3.y),
    ];

    let mut values = [
        EdgePoint { distance: d1, point: p1, edge: e1 },
        EdgePoint { distance: d2, point: p2, edge: e2 },
        EdgePoint { distance: d3, point: p3, edge: e3 },
    ];

    if values[0].distance > values[1].distance {
        values.swap(0, 1);
    }

    if values[1].distance > values[2].distance {
        values.swap(1, 2);
    }

    if values[0].distance > values[1].distance {
        values.swap(0, 1);
    }

    let nearest = values[0];
    let furthest = values[2];
    [
        (nearest.distance, nearest.point, nearest.edge as u32),
        (nearest.distance, nearest.point, nav.previous_edge(nearest.edge) as u32),
        (furthest.distance, furthest.point, nav.next_edge(nearest.edge) as u32),
    ]
}

fn orient2d(p1: Position<f32>, p2: Position<f32>, p3: Position<f32>) -> f64 {
    use robust::Coord;
    robust::orient2d(
        Coord { x: p1.x, y: p1.y },
        Coord { x: p2.x, y: p2.y },
        Coord { x: p3.x, y: p3.y },
    )
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

