mod terrain;
use terrain::Terrain;

use crate::assets::{AnimationBase, DecorationBase, ResourceBase, StructureBase, Texture};
use crate::error::Error;
use crate::shared::AABB;
use crate::store::SaveAndLoad;
use crate::Position;

#[derive(Copy, Clone, Default, Debug)]
pub struct BaseUnit {
    pub position: Position<f32>,
    pub animation: AnimationBase,
    pub current_frame: u8,
    pub flipped: bool,
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

    pub terrain: Terrain,

    pub pawn_texture: Texture,
    pub pawns: Vec<BaseUnit>,

    pub warrior_texture: Texture,
    pub warriors: Vec<BaseUnit>,

    pub archer_texture: Texture,
    pub archers: Vec<BaseUnit>,

    pub torch_goblin_texture: Texture,
    pub torch_goblins: Vec<BaseUnit>,

    pub tnt_goblin_texture: Texture,
    pub tnt_goblins: Vec<BaseUnit>,

    pub sheep_texture: Texture,
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
        let textures = [
            (&mut self.pawn_texture, "pawn"),
            (&mut self.warrior_texture, "warrior"),
            (&mut self.archer_texture, "archer"),
            (&mut self.torch_goblin_texture, "torch_goblin"),
            (&mut self.tnt_goblin_texture, "tnt_goblin"),
            (&mut self.sheep_texture, "sheep"),
        ];

        for (texture, name) in textures {
            *texture = assets.textures.get(name).copied()
                .ok_or_else(|| assets_err!("{} texture missing", name) )?;
        }

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

    pub fn create_tnt_goblin(&mut self, position: Position<f32>, animation: &AnimationBase) -> usize {
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
        writer.write(&self.pawn_texture);
        writer.write_slice(&self.pawns);

        writer.write(&self.warrior_texture);
        writer.write_slice(&self.warriors);

        writer.write(&self.archer_texture);
        writer.write_slice(&self.archers);

        writer.write(&self.torch_goblin_texture);
        writer.write_slice(&self.torch_goblins);

        writer.write(&self.tnt_goblin_texture);
        writer.write_slice(&self.tnt_goblins);

        writer.write(&self.sheep_texture);
        writer.write_slice(&self.sheeps);

        writer.write_slice(&self.decorations);
        writer.write_slice(&self.structures);
        writer.write_slice(&self.resources);

        writer.write(&self.static_resources_texture);
        writer.write_u32(self.total_sprite_count);

        writer.save(&self.terrain);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let pawn_texture = reader.read();
        let pawns = reader.read_slice().to_vec();

        let warrior_texture = reader.read();
        let warriors = reader.read_slice().to_vec();

        let archer_texture = reader.read();
        let archers = reader.read_slice().to_vec();

        let torch_goblin_texture = reader.read();
        let torch_goblins = reader.read_slice().to_vec();

        let tnt_goblin_texture = reader.read();
        let tnt_goblins = reader.read_slice().to_vec();

        let sheep_texture = reader.read();
        let sheeps = reader.read_slice().to_vec();

        let decorations = reader.read_slice().to_vec();
        let structures = reader.read_slice().to_vec();
        let resources = reader.read_slice().to_vec();

        let static_resources_texture = reader.read();
        let total_sprite_count = reader.read_u32();

        let terrain = reader.load();

        World {
            last_animation_tick: 0.0,
            total_sprite_count,
            static_resources_texture,

            terrain,

            pawn_texture,
            pawns,

            warrior_texture,
            warriors,

            archer_texture,
            archers,

            torch_goblin_texture,
            torch_goblins,

            tnt_goblin_texture,
            tnt_goblins,

            sheep_texture,
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

            terrain: Terrain::default(),
    
            pawn_texture: Texture { id: 0 },
            pawns: Vec::with_capacity(16),

            warrior_texture: Texture { id: 0 },
            warriors: Vec::with_capacity(16),

            archer_texture: Texture { id: 0 },
            archers: Vec::with_capacity(16),

            torch_goblin_texture: Texture { id: 0 },
            torch_goblins: Vec::with_capacity(16),

            tnt_goblin_texture: Texture { id: 0 },
            tnt_goblins: Vec::with_capacity(16),

            sheep_texture: Texture { id: 0 },
            sheeps: Vec::with_capacity(16),

            decorations: Vec::with_capacity(16),
            structures: Vec::with_capacity(16),
            resources: Vec::with_capacity(16)
        }
    }
}
