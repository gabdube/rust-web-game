//! Generate the navmesh for the terrain

use crate::pathfinding::PathfindingState;
use crate::world::Terrain;
use super::World;


pub(super) fn generate(world: &mut World) {
    world.pathfinding.navmesh.clear();
    terrain_limits(&world.terrain, &mut world.pathfinding);
    structures_collisions(world);
    world.pathfinding.navmesh.generate();
}

fn terrain_limits(terrain: &Terrain, pathfinding: &mut PathfindingState) {
    let navmesh = &mut pathfinding.navmesh;
    let right = terrain.max_width_pixel() as f32;
    let bottom = terrain.max_height_pixel() as f32;

    navmesh.push_point(0.0, 0.0);
    navmesh.push_point(0.0, bottom);
    navmesh.push_point(right, 0.0);
    navmesh.push_point(right, bottom);
}

fn structures_collisions(world: &mut World) {
    let structures = &world.structures;
    let navmesh = &mut world.pathfinding.navmesh;

    for structure in structures {
        navmesh.push_aabb(structure.aabb());
    }
}
