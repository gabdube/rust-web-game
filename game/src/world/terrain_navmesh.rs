//! Generate the navmesh for the terrain
use crate::pathfinding::PathfindingState;
use crate::world::Terrain;


pub(super) fn generate_terrain_navmesh(terrain: &Terrain, pathfinding: &mut PathfindingState) {
    pathfinding.clear_terrain_bounds();
    terrain_limits(terrain, pathfinding);
}

fn terrain_limits(terrain: &Terrain, pathfinding: &mut PathfindingState) {
    let right = terrain.max_width_pixel() as f32;
    let bottom = terrain.max_height_pixel() as f32;
    
    pathfinding.add_terrain_bound(0.0, 0.0);
    pathfinding.add_terrain_bound(0.0, bottom);
    pathfinding.add_terrain_bound(right, 0.0);
    pathfinding.add_terrain_bound(right, bottom);
}
