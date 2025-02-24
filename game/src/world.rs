mod terrain;
use terrain::Terrain;

use crate::assets::{AnimationBase, Texture};
use crate::error::Error;
use crate::output::SpriteData;
use crate::store::SaveAndLoad;
use crate::Position;


#[derive(Copy, Clone, Default, Debug)]
pub struct Pawn {
    pub position: Position<f32>,
    pub animation: AnimationBase,
    pub current_frame: u8,
    pub flipped: bool,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Warrior {
    pub position: Position<f32>,
    pub animation: AnimationBase,
    pub current_frame: u8,
    pub flipped: bool,
}

/// The game world data. Includes actors, terrain, and decorations
pub struct World {
    pub last_animation_tick: f64,
    pub terrain: Terrain,

    pub pawn_texture: Texture,
    pub pawns: Vec<Pawn>,
    pub pawns_sprites: Vec<SpriteData>,

    pub warrior_texture: Texture,
    pub warriors: Vec<Warrior>,
    pub warrior_sprites: Vec<SpriteData>,
}

impl World {

    pub fn init_assets(&mut self, assets: &crate::assets::Assets) -> Result<(), Error> {
        self.pawn_texture = assets.textures.get("pawn").copied()
            .ok_or_else(|| assets_err!("Pawn texture missing") )?;
    
        self.warrior_texture = assets.textures.get("warrior").copied()
            .ok_or_else(|| assets_err!("Warrior texture missing") )?;

        Ok(())
    }

    pub fn reset(&mut self) {
        self.pawns.clear();
        self.pawns_sprites.clear();
        self.warriors.clear();
        self.warrior_sprites.clear();
        self.terrain.reset();
    }

    pub fn init_terrain(&mut self, width: u32, height: u32) {
        self.terrain.init_terrain(width, height);
    }

    pub fn create_pawn(&mut self, position: &Position<f32>, animation: &AnimationBase) -> usize {
        let pawn_index = self.pawns.len();

        self.pawns_sprites.push(Self::build_sprite_data(position, animation, 0, false));

        self.pawns.push(Pawn {
            position: *position,
            animation: *animation,
            ..Default::default()
        });

        pawn_index
    }

    pub fn create_warrior(&mut self, position: &Position<f32>, animation: &AnimationBase) -> usize {
        let warrior_index = self.warriors.len();

        self.warrior_sprites.push(Self::build_sprite_data(position, animation, 0, false));

        self.warriors.push(Warrior {
            position: *position,
            animation: *animation,
            ..Default::default()
        });

        warrior_index
    }

    pub fn inner_animation_update(&mut self) {
        for (index, pawn) in self.pawns.iter_mut().enumerate() {
            pawn.current_frame += 1;
            if pawn.current_frame > pawn.animation.last_frame {
                pawn.current_frame = 0;
            }

            self.pawns_sprites[index] = Self::build_sprite_data(&pawn.position, &pawn.animation, pawn.current_frame, pawn.flipped);
        }

        for (index, warrior) in self.warriors.iter_mut().enumerate() {
            warrior.current_frame += 1;
            if warrior.current_frame > warrior.animation.last_frame {
                warrior.current_frame = 0;
            }

            self.warrior_sprites[index] = Self::build_sprite_data(&warrior.position, &warrior.animation, warrior.current_frame, warrior.flipped);
        }
    } 

    fn build_sprite_data(position: &Position<f32>, animation: &AnimationBase, current_frame: u8, flipped: bool) -> SpriteData {
        let mut sprite = SpriteData::default();
        let i = current_frame as f32;
        sprite.position[0] = position.x - (animation.sprite_width * 0.5);
        sprite.position[1] = position.y - (animation.sprite_height * 0.5);
        sprite.size[0] = animation.sprite_width;
        sprite.size[1] = animation.sprite_height;
        sprite.texcoord_offset[0] = animation.x + (animation.sprite_width * i) + (animation.padding * i);
        sprite.texcoord_offset[1] = animation.y;
        sprite.texcoord_size[0] = animation.sprite_width;
        sprite.texcoord_size[1] = animation.sprite_height;

        if flipped {
            sprite.texcoord_offset[0] += sprite.size[0];
            sprite.texcoord_size[0] *= -1.0;
        }

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
        writer.save(&self.terrain);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let pawn_texture = reader.read();
        let pawns = reader.read_slice().to_vec();
        let pawns_sprites = reader.read_slice().to_vec();

        let warrior_texture = reader.read();
        let warriors = reader.read_slice().to_vec();
        let warrior_sprites = reader.read_slice().to_vec();

        let terrain = reader.load();

        World {
            last_animation_tick: 0.0,
            terrain,

            pawn_texture,
            pawns,
            pawns_sprites,

            warrior_texture,
            warriors,
            warrior_sprites,
        }
    }

}

impl Default for World {
    fn default() -> Self {
        World {
            last_animation_tick: 0.0,
            terrain: Terrain::default(),
    
            pawn_texture: Texture { id: 0 },
            pawns: Vec::with_capacity(16),
            pawns_sprites: Vec::with_capacity(16),

            warrior_texture: Texture { id: 0 },
            warriors: Vec::with_capacity(16),
            warrior_sprites: Vec::with_capacity(16),
        }
    }
}
