//! AStar pathfinding over a `NavMesh`
use fnv::FnvHashMap;
use std::collections::BinaryHeap;
use std::cmp::Ord;
use crate::shared::Position;
use super::navmesh::{NavMesh, NavTriangle};

#[derive(Copy, Clone, Default)]
struct EdgeHeuristic {
    point: Position<f32>,
    cost: f32,
    heuristic: f32,
    edge: usize,
}

struct NavCell {
    triangle: NavTriangle,
    edge: u32,
    position: Position<f32>,
    estimated_cost: f32,
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

fn reverse_path(
    parents: &FnvHashMap<NavTriangle, (NavTriangle, u32, f32)>,
    edges: &mut Vec<u32>,
    start: NavTriangle,
    start_edge: u32,
) {
    let mut last = start;
    loop {
        let (triangle, edge, _) = match parents.get(&last).copied() {
            Some(value) => value,
            None => { break; }
        };

        if edge == u32::MAX {
            break;
        }

        edges.push(edge);

        last = triangle;
    }

    edges.reverse();
}

pub(super) fn debug_path(
    edges: &mut Vec<u32>,
    nav: &NavMesh,
    start_triangle: NavTriangle,
    end_triangle: NavTriangle,
    start: Position<f32>,
    end: Position<f32>
) -> bool  {
    use std::collections::hash_map::Entry;

    let mut to_see: BinaryHeap<NavCell> = BinaryHeap::new();
    let mut parents = FnvHashMap::default();

    to_see.push(NavCell { triangle: start_triangle, edge: u32::MAX, position: start, estimated_cost: 0.0, cost: 0.0 });
    parents.insert(start_triangle, (start_triangle, u32::MAX, 0.0)); // triangle: (parent triangle, connecting_edge, min_distance)

    while let Some(cell) = to_see.pop() {
        if cell.triangle == end_triangle {
            reverse_path(&parents, edges, cell.triangle, cell.edge);
            return true;
        }

        let mut edges: [EdgeHeuristic; 3] = Default::default();
        build_edges(nav, cell.triangle, cell.cost, start, end, &mut edges);
        choose_edges(nav, &mut edges);

        for h_edge in edges {
            if cell.edge == (h_edge.edge as u32) {
                continue;
            }

            let neighbor_edge = nav.triangulation.halfedges[h_edge.edge];
            if neighbor_edge == usize::MAX {
                continue;
            }

            let neigbours_triangle = nav.triangle_of_edge(neighbor_edge);
            match parents.entry(neigbours_triangle) {
                Entry::Vacant(e) => {
                    e.insert((cell.triangle, neighbor_edge as u32, h_edge.cost));
                }
                Entry::Occupied(mut e) => {
                    if e.get().2 > h_edge.cost {
                        e.insert((cell.triangle, neighbor_edge as u32, h_edge.cost));
                    } else {
                        continue;
                    }
                }
            }

            to_see.push(NavCell {
                triangle: neigbours_triangle,
                edge: neighbor_edge as u32,
                position: h_edge.point,
                estimated_cost: h_edge.heuristic,
                cost: h_edge.cost
            });
        }
    }

    false
}

//
// Helpers
//

fn build_edges(nav: &NavMesh, triangle: NavTriangle, base_cost: f32, start: Position<f32>, end: Position<f32>, edges: &mut [EdgeHeuristic; 3]) {
    let triangle_index = triangle.index();
    let [e1, e2, e3] = [3*triangle_index+0, 3*triangle_index+1, 3*triangle_index+2];
    let [p1, p2, p3] = [nav.point_of_edge(e1), nav.point_of_edge(e2), nav.point_of_edge(e3)];
    let [c1, c2, c3] = [base_cost + heuristic(start, p1), base_cost + heuristic(start, p2), base_cost + heuristic(start, p3)];
    let [h1, h2, h3] = [c1 + heuristic(end, p1), c2 + heuristic(end, p2), c3 + heuristic(end, p3)];
    *edges = [
        EdgeHeuristic { point: p1, cost: c1, heuristic: h1, edge: e1 },
        EdgeHeuristic { point: p2, cost: c2, heuristic: h2, edge: e2 },
        EdgeHeuristic { point: p3, cost: c3, heuristic: h3, edge: e3 },
    ];
} 

fn choose_edges(nav: &NavMesh, edges: &mut [EdgeHeuristic; 3]) {
    if edges[0].heuristic > edges[1].heuristic {
        edges.swap(0, 1);
    }

    if edges[1].heuristic > edges[2].heuristic {
        edges.swap(1, 2);
    }

    if edges[0].heuristic > edges[1].heuristic {
        edges.swap(0, 1);
    }

    let nearest = edges[0];
    let opposed = edges[1];

    *edges = [
        EdgeHeuristic { point: nearest.point, cost: nearest.cost, heuristic: nearest.heuristic, edge: nearest.edge },
        EdgeHeuristic { point: nearest.point, cost: nearest.cost, heuristic: nearest.heuristic, edge: nav.previous_edge(nearest.edge)},
        EdgeHeuristic { point: opposed.point, cost: opposed.cost, heuristic: opposed.heuristic, edge: nav.next_edge(nearest.edge) },
    ];
}

#[inline(always)]
fn heuristic(p1: Position<f32>, p2: Position<f32>) -> f32 {
    f32::abs(p1.x - p2.x) + f32::abs(p1.y - p2.y)
}

//
// Other Impl
//

impl Ord for NavCell {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match other.estimated_cost.total_cmp(&self.estimated_cost) {
            std::cmp::Ordering::Equal => self.cost.total_cmp(&other.cost),
            s => s,
        }
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
