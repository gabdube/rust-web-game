mod terrain;
use terrain::Terrain;

use crate::assets::{AnimationBase, Texture, DecorationBase};
use crate::error::Error;
use crate::output::SpriteData;
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
}

/// The game world data. Includes actors, terrain, and decorations
pub struct World {
    pub last_animation_tick: f64,
    pub static_resources_texture: Texture,

    pub terrain: Terrain,

    pub pawn_texture: Texture,
    pub pawns: Vec<BaseUnit>,
    pub pawns_sprites: Vec<SpriteData>,

    pub warrior_texture: Texture,
    pub warriors: Vec<BaseUnit>,
    pub warrior_sprites: Vec<SpriteData>,

    pub archer_texture: Texture,
    pub archers: Vec<BaseUnit>,
    pub archer_sprites: Vec<SpriteData>,

    pub torch_goblin_texture: Texture,
    pub torch_goblins: Vec<BaseUnit>,
    pub torch_goblins_sprites: Vec<SpriteData>,

    pub tnt_goblin_texture: Texture,
    pub tnt_goblins: Vec<BaseUnit>,
    pub tnt_goblins_sprites: Vec<SpriteData>,

    pub sheep_texture: Texture,
    pub sheeps: Vec<BaseUnit>,
    pub sheep_sprites: Vec<SpriteData>,

    pub decorations: Vec<BaseStatic>,
    pub decoration_sprites: Vec<SpriteData>
}

impl World {

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
        self.pawns_sprites.clear();

        self.warriors.clear();
        self.warrior_sprites.clear();

        self.archers.clear();
        self.archer_sprites.clear();

        self.torch_goblins.clear();
        self.torch_goblins_sprites.clear();

        self.tnt_goblins.clear();
        self.tnt_goblins_sprites.clear();

        self.sheeps.clear();
        self.sheep_sprites.clear();

        self.decorations.clear();
        self.decoration_sprites.clear();

        self.terrain.reset();
    }

    pub fn init_terrain(&mut self, width: u32, height: u32) {
        self.terrain.init_terrain(width, height);
    }

    pub fn animation_update(&mut self) {
        let groups = [
            (&mut self.pawns, &mut self.pawns_sprites),
            (&mut self.warriors, &mut self.warrior_sprites),
            (&mut self.archers, &mut self.archer_sprites),
            (&mut self.torch_goblins, &mut self.torch_goblins_sprites),
            (&mut self.tnt_goblins, &mut self.tnt_goblins_sprites),
            (&mut self.sheeps, &mut self.sheep_sprites),
        ];

        for (actors, sprites) in groups {
            for (index, actor) in actors.iter_mut().enumerate() {
                actor.current_frame += 1;
                if actor.current_frame > actor.animation.last_frame {
                    actor.current_frame = 0;
                }
    
                sprites[index] = Self::build_sprite_data(&actor.position, &actor.animation, actor.current_frame, actor.flipped);
            }
        }
    } 

    pub fn create_pawn(&mut self, position: &Position<f32>, animation: &AnimationBase) -> usize {
        Self::create_inner_actor(&mut self.pawns, &mut self.pawns_sprites, position, animation)
    }

    pub fn create_warrior(&mut self, position: &Position<f32>, animation: &AnimationBase) -> usize {
        Self::create_inner_actor(&mut self.warriors, &mut self.warrior_sprites, position, animation)
    }

    pub fn create_archer(&mut self, position: &Position<f32>, animation: &AnimationBase) -> usize {
        Self::create_inner_actor(&mut self.archers, &mut self.archer_sprites, position, animation)
    }

    pub fn create_torch_goblin(&mut self, position: &Position<f32>, animation: &AnimationBase) -> usize {
        Self::create_inner_actor(&mut self.torch_goblins, &mut self.torch_goblins_sprites, position, animation)
    }

    pub fn create_tnt_goblin(&mut self, position: &Position<f32>, animation: &AnimationBase) -> usize {
        Self::create_inner_actor(&mut self.tnt_goblins, &mut self.tnt_goblins_sprites, position, animation)
    }
    
    pub fn create_sheep(&mut self, position: &Position<f32>, animation: &AnimationBase) -> usize {
        Self::create_inner_actor(&mut self.sheeps, &mut self.sheep_sprites, position, animation)
    }

    pub fn create_decoration(&mut self, position: &Position<f32>, deco: &DecorationBase) -> usize {
        let index = self.decorations.len();
        self.decoration_sprites.push(Self::build_static_sprite_data(position, &deco.aabb));
        self.decorations.push(BaseStatic { position: *position });
        index
    }

    fn create_inner_actor(
        base: &mut Vec<BaseUnit>,
        sprites: &mut Vec<SpriteData>,
        position: &Position<f32>,
        animation: &AnimationBase
    ) -> usize {
        let index = base.len();
        sprites.push(Self::build_sprite_data(position, animation, 0, false));
        base.push(BaseUnit { position: *position, animation: *animation, ..Default::default()});
        return index
    }

    fn build_sprite_data(position: &Position<f32>, animation: &AnimationBase, current_frame: u8, flipped: bool) -> SpriteData {
        let mut sprite = SpriteData::default();
        let i = current_frame as f32;
        sprite.position[0] = position.x - (animation.sprite_width * 0.5);
        sprite.position[1] = position.y - animation.sprite_height;
        sprite.size[0] = animation.sprite_width;
        sprite.size[1] = animation.sprite_height;
        sprite.texcoord_offset[0] = animation.x + (animation.sprite_width * i) + (animation.padding * i);
        sprite.texcoord_offset[1] = animation.y;
        sprite.texcoord_size[0] = sprite.size[0];
        sprite.texcoord_size[1] = sprite.size[1];

        if flipped {
            sprite.texcoord_offset[0] += sprite.size[0];
            sprite.texcoord_size[0] *= -1.0;
        }

        sprite
    }

    fn build_static_sprite_data(position: &Position<f32>, sprite_rect: &AABB) -> SpriteData {
        let mut sprite = SpriteData::default();
        sprite.position[0] = position.x - (sprite_rect.width() * 0.5);
        sprite.position[1] = position.y - sprite_rect.height();
        sprite.size[0] = sprite_rect.width();
        sprite.size[1] = sprite_rect.height();
        sprite.texcoord_offset[0] = sprite_rect.left;
        sprite.texcoord_offset[1] = sprite_rect.top;
        sprite.texcoord_size[0] = sprite.size[0];
        sprite.texcoord_size[1] = sprite.size[1];

        sprite
    }

}

impl SaveAndLoad for World {

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write(&self.pawn_texture);
        writer.write_slice(&self.pawns);
        writer.write_slice(&self.pawns_sprites);

        writer.write(&self.warrior_texture);
        writer.write_slice(&self.warriors);
        writer.write_slice(&self.warrior_sprites);

        writer.write(&self.archer_texture);
        writer.write_slice(&self.archers);
        writer.write_slice(&self.archer_sprites);

        writer.write(&self.torch_goblin_texture);
        writer.write_slice(&self.torch_goblins);
        writer.write_slice(&self.torch_goblins_sprites);

        writer.write(&self.tnt_goblin_texture);
        writer.write_slice(&self.tnt_goblins);
        writer.write_slice(&self.tnt_goblins_sprites);

        writer.write(&self.sheep_texture);
        writer.write_slice(&self.sheeps);
        writer.write_slice(&self.sheep_sprites);

        writer.write_slice(&self.decorations);
        writer.write_slice(&self.decoration_sprites);

        writer.write(&self.static_resources_texture);

        writer.save(&self.terrain);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let pawn_texture = reader.read();
        let pawns = reader.read_slice().to_vec();
        let pawns_sprites = reader.read_slice().to_vec();

        let warrior_texture = reader.read();
        let warriors = reader.read_slice().to_vec();
        let warrior_sprites = reader.read_slice().to_vec();

        let archer_texture = reader.read();
        let archers = reader.read_slice().to_vec();
        let archer_sprites = reader.read_slice().to_vec();

        let torch_goblin_texture = reader.read();
        let torch_goblins = reader.read_slice().to_vec();
        let torch_goblins_sprites = reader.read_slice().to_vec();

        let tnt_goblin_texture = reader.read();
        let tnt_goblins = reader.read_slice().to_vec();
        let tnt_goblins_sprites = reader.read_slice().to_vec();

        let sheep_texture = reader.read();
        let sheeps = reader.read_slice().to_vec();
        let sheep_sprites = reader.read_slice().to_vec();

        let decorations = reader.read_slice().to_vec();
        let decoration_sprites = reader.read_slice().to_vec();

        let static_resources_texture = reader.read();

        let terrain = reader.load();

        World {
            last_animation_tick: 0.0,
            static_resources_texture,

            terrain,

            pawn_texture,
            pawns,
            pawns_sprites,

            warrior_texture,
            warriors,
            warrior_sprites,

            archer_texture,
            archers,
            archer_sprites,

            torch_goblin_texture,
            torch_goblins,
            torch_goblins_sprites,

            tnt_goblin_texture,
            tnt_goblins,
            tnt_goblins_sprites,

            sheep_texture,
            sheeps,
            sheep_sprites,

            decorations,
            decoration_sprites,
        }
    }

}

impl Default for World {
    fn default() -> Self {
        World {
            last_animation_tick: 0.0,
            static_resources_texture: Texture { id: 0 },

            terrain: Terrain::default(),
    
            pawn_texture: Texture { id: 0 },
            pawns: Vec::with_capacity(16),
            pawns_sprites: Vec::with_capacity(16),

            warrior_texture: Texture { id: 0 },
            warriors: Vec::with_capacity(16),
            warrior_sprites: Vec::with_capacity(16),

            archer_texture: Texture { id: 0 },
            archers: Vec::with_capacity(16),
            archer_sprites: Vec::with_capacity(16),

            torch_goblin_texture: Texture { id: 0 },
            torch_goblins: Vec::with_capacity(16),
            torch_goblins_sprites: Vec::with_capacity(16),

            tnt_goblin_texture: Texture { id: 0 },
            tnt_goblins: Vec::with_capacity(16),
            tnt_goblins_sprites: Vec::with_capacity(16),

            sheep_texture: Texture { id: 0 },
            sheeps: Vec::with_capacity(16),
            sheep_sprites: Vec::with_capacity(16),

            decorations: Vec::with_capacity(16),
            decoration_sprites: Vec::with_capacity(16),
        }
    }
}
