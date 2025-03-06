mod terrain;
use terrain::Terrain;

use crate::assets::{AnimationBase, DecorationBase, ResourceBase, StructureBase, Texture};
use crate::error::Error;
use crate::shared::{AABB, aabb, size};
use crate::store::SaveAndLoad;
use crate::Position;

#[derive(Debug)]
pub enum WorldObjectType {
    Pawn,
    Warrior,
    Archer,
    TorchGoblin,
    DynamiteGoblin,
    Sheep,
    Decoration,
    Structure,
    Resource
}

#[derive(Debug)]
pub struct WorldObject {
    id: u32,
    ty: WorldObjectType,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct BaseUnit {
    pub position: Position<f32>,
    pub animation: AnimationBase,
    pub current_frame: u8,
    pub flipped: bool,
}

impl BaseUnit {
    pub const fn aabb(&self) -> AABB {
        let mut position = self.position;
        let size = size(self.animation.sprite_width, self.animation.sprite_height);
        position.x -= size.width * 0.5;
        position.y -= size.height;
        aabb(position, size)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct BaseStatic {
    pub position: Position<f32>,
    pub aabb: AABB,
}

/// The game world data. Includes actors, terrain, and decorations
pub struct World {
    pub last_animation_tick: f64,
    pub total_sprite_count: u32,

    pub static_resources_texture: Texture,
    pub units_texture: Texture,

    pub terrain: Terrain,
    pub pawns: Vec<BaseUnit>,
    pub warriors: Vec<BaseUnit>,
    pub archers: Vec<BaseUnit>,
    pub torch_goblins: Vec<BaseUnit>,
    pub tnt_goblins: Vec<BaseUnit>,
    pub sheeps: Vec<BaseUnit>,

    pub decorations: Vec<BaseStatic>,
    pub structures: Vec<BaseStatic>,
    pub resources: Vec<BaseStatic>,
}

impl World {

    pub fn total_sprites(&mut self) -> usize {
        self.total_sprite_count as usize
    }

    pub fn init_assets(&mut self, assets: &crate::assets::Assets) -> Result<(), Error> {
        self.units_texture = assets.textures.get("units").copied()
            .ok_or_else(|| assets_err!("units texture missing") )?;

        self.static_resources_texture = assets.textures.get("static_resources").copied()
            .ok_or_else(|| assets_err!("static_resources texture missing") )?;

        Ok(())
    }

    pub fn reset(&mut self) {
        self.pawns.clear();
        self.warriors.clear();
        self.archers.clear();
        self.torch_goblins.clear();
        self.tnt_goblins.clear();
        self.sheeps.clear();
        self.decorations.clear();
        self.structures.clear();
        self.resources.clear();
        self.terrain.reset();
    }

    pub fn init_terrain(&mut self, width: u32, height: u32) {
        self.terrain.init_terrain(width, height);
    }

    pub fn create_pawn(&mut self, position: Position<f32>, animation: &AnimationBase) -> usize {
        self.total_sprite_count += 1;
        Self::create_inner_actor(&mut self.pawns, position, animation)
    }

    pub fn update_pawn_position(&mut self, pawn_id: usize, position: Position<f32>) {
        self.pawns[pawn_id].position = position;
    }

    pub fn create_warrior(&mut self, position: Position<f32>, animation: &AnimationBase) -> usize {
        self.total_sprite_count += 1;
        Self::create_inner_actor(&mut self.warriors, position, animation)
    }

    pub fn create_archer(&mut self, position: Position<f32>, animation: &AnimationBase) -> usize {
        self.total_sprite_count += 1;
        Self::create_inner_actor(&mut self.archers, position, animation)
    }

    pub fn create_torch_goblin(&mut self, position: Position<f32>, animation: &AnimationBase) -> usize {
        self.total_sprite_count += 1;
        Self::create_inner_actor(&mut self.torch_goblins, position, animation)
    }

    pub fn create_dynamite_goblin(&mut self, position: Position<f32>, animation: &AnimationBase) -> usize {
        self.total_sprite_count += 1;
        Self::create_inner_actor(&mut self.tnt_goblins, position, animation)
    }
    
    pub fn create_sheep(&mut self, position: Position<f32>, animation: &AnimationBase) -> usize {
        self.total_sprite_count += 1;
        Self::create_inner_actor(&mut self.sheeps, position, animation)
    }

    pub fn create_decoration(&mut self, position: Position<f32>, deco: &DecorationBase) -> usize {
        self.total_sprite_count += 1;
        let index = self.decorations.len();
        self.decorations.push(BaseStatic { position, aabb: deco.aabb });
        index
    }

    pub fn create_structure(&mut self, position: Position<f32>, structure: &StructureBase) -> usize {
        self.total_sprite_count += 1;
        let index = self.structures.len();
        self.structures.push(BaseStatic { position, aabb: structure.aabb });
        index
    }

    pub fn create_resource(&mut self, position: Position<f32>, resource: &ResourceBase) -> usize {
        self.total_sprite_count += 1;
        let index = self.resources.len();
        self.resources.push(BaseStatic { position, aabb: resource.aabb });
        index
    }

    pub fn actor_at(&self, position: Position<f32>) -> Option<WorldObject> {
        let types = [
            WorldObjectType::Pawn,
            WorldObjectType::Warrior,
            WorldObjectType::Archer,
            WorldObjectType::TorchGoblin,
            WorldObjectType::DynamiteGoblin,
            WorldObjectType::Sheep,
        ];
        let groups = [
            &self.pawns,
            &self.warriors,
            &self.archers,
            &self.torch_goblins,
            &self.tnt_goblins,
            &self.sheeps,
        ];

        for (group, ty) in groups.into_iter().zip(types) {
            for (id, actor) in group.iter().enumerate() {
                if actor.aabb().point_inside(position) {
                    return Some(WorldObject { id: id as u32, ty })
                }
            }
        }

        None
    }

    pub fn object_at(&self, position: Position<f32>) -> Option<WorldObject> {
        if let Some(actor) = self.actor_at(position) {
            return Some(actor);
        }

        None
    }

    fn create_inner_actor(
        base: &mut Vec<BaseUnit>,
        position: Position<f32>,
        animation: &AnimationBase
    ) -> usize {
        let index = base.len();
        base.push(BaseUnit { position, animation: *animation, ..Default::default()});
        return index
    }

}

impl SaveAndLoad for World {

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.pawns);
        writer.write_slice(&self.warriors);
        writer.write_slice(&self.archers);
        writer.write_slice(&self.torch_goblins);
        writer.write_slice(&self.tnt_goblins);
        writer.write_slice(&self.sheeps);
        writer.write_slice(&self.decorations);
        writer.write_slice(&self.structures);
        writer.write_slice(&self.resources);

        writer.write(&self.static_resources_texture);
        writer.write(&self.units_texture);
        writer.write_u32(self.total_sprite_count);

        writer.save(&self.terrain);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let pawns = reader.read_slice().to_vec();
        let warriors = reader.read_slice().to_vec();
        let archers = reader.read_slice().to_vec();
        let torch_goblins = reader.read_slice().to_vec();
        let tnt_goblins = reader.read_slice().to_vec();
        let sheeps = reader.read_slice().to_vec();

        let decorations = reader.read_slice().to_vec();
        let structures = reader.read_slice().to_vec();
        let resources = reader.read_slice().to_vec();

        let static_resources_texture = reader.read();
        let units_texture = reader.read();
        let total_sprite_count = reader.read_u32();

        let terrain = reader.load();

        World {
            last_animation_tick: 0.0,
            total_sprite_count,
            static_resources_texture,
            units_texture,

            terrain,

            pawns,
            warriors,
            archers,
            torch_goblins,
            tnt_goblins,
            sheeps,

            decorations,
            structures,
            resources,
        }
    }

}

impl Default for World {
    fn default() -> Self {
        World {
            last_animation_tick: 0.0,
            total_sprite_count: 0,
            static_resources_texture: Texture { id: 0 },
            units_texture: Texture { id: 0 },

            terrain: Terrain::default(),
    
            pawns: Vec::with_capacity(16),
            warriors: Vec::with_capacity(16),
            archers: Vec::with_capacity(16),
            torch_goblins: Vec::with_capacity(16),
            tnt_goblins: Vec::with_capacity(16),
            sheeps: Vec::with_capacity(16),

            decorations: Vec::with_capacity(16),
            structures: Vec::with_capacity(16),
            resources: Vec::with_capacity(16)
        }
    }
}
