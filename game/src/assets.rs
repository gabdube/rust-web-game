mod animations;
pub use animations::AnimationBase;

mod terrain_tilemap;
pub use terrain_tilemap::{TerrainTilemap, TerrainCell};

mod decoration;
pub use decoration::*;

mod structures;
pub use structures::*;

mod resources;
pub use resources::*;

use animations::AnimationsBundle;
use fnv::FnvHashMap;
use crate::error::Error;

#[derive(Copy, Clone)]
pub struct Texture {
    // The unique ID of the texture. It is also the index of the texture array in the engine data
    pub id: u32,
}

#[derive(Default)]
pub struct Assets {
    pub textures: FnvHashMap<String, Texture>,
    pub terrain: TerrainTilemap,
    pub decorations: DecorationBundle,
    pub structures: StructuresBundle,
    pub resources: ResourcesBundle,
    pub animations: AnimationsBundle,
}

impl Assets {

    pub fn load_bundle(&mut self, init: &crate::DemoGameInit) -> Result<(), Error> {
        let mut error: Option<Error> = None;

        // Assets index
        crate::shared::split_csv::<5, _>(&init.assets_bundle, |args| {
            let result = match args[0] {
                "TEXTURE" => {
                    self.load_texture(args)
                },
                "CSV" => {
                    self.load_csv(init, args)
                },
                "SHADER" => Ok(()),
                _ => { Err(assets_err!("Unknown asset type {:?}", args[0])) }
            };

            if let Err(new_error) = result {
                crate::shared::merge_error(&mut error, new_error)
            }
        });

        if let Some(err) = error {
            return Err(err);
        }

        Ok(())
    }

    fn load_texture(&mut self, args: &[&str]) -> Result<(), Error> {
        let name = args.get(1)
            .map(|value| value.to_string() )
            .ok_or_else(|| assets_err!("Missing texture name") )?;

        let id = self.textures.len() as u32;
        self.textures.insert(name, Texture { id });

        Ok(())
    }

    fn load_csv(&mut self, init: &crate::DemoGameInit, args: &[&str]) -> Result<(), Error> {
        let &csv_name = args.get(1)
            .ok_or_else(|| assets_err!("Missing csv name") )?;

        let csv_string = init.text_assets.get(csv_name)
            .ok_or_else(|| assets_err!("Failed to match csv name to csv data") )?;
        
        match csv_name {
            "terrain_sprites" => {
                self.terrain.load(csv_string.as_str())?;
            },
            "static_sprites" => {
                let sprites = csv_string.as_str();
                self.decorations.load(sprites);
                self.structures.load(sprites);
                self.resources.load(sprites);
            },
            "units_sprites" => {
                let sprites = csv_string.as_str();
                self.animations.load_animations(sprites)?;
            },
            name => {
                warn!("Unknown csv: {:?}", name);
            }
        }

        Ok(())
    }
}

impl crate::store::SaveAndLoad for Assets {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_string_hashmap(&self.textures);
        writer.save(&self.terrain);
        writer.write(&self.decorations);
        writer.write(&self.structures);
        writer.write(&self.resources);
        writer.write(&self.animations);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let mut assets = Assets::default();
        assets.textures = reader.read_string_hashmap();
        assets.terrain = reader.load();
        assets.decorations = reader.read();
        assets.structures = reader.read();
        assets.resources = reader.read();
        assets.animations = reader.read();
        assets
    }
}
