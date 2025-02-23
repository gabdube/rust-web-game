mod animations;
pub use animations::AnimationBase;

mod terrain_tilemap;
pub use terrain_tilemap::{TerrainTilemap, TerrainCell};

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
    pub animations: AnimationsBundle,
}

impl Assets {

    pub fn load_bundle(&mut self, init: &crate::DemoGameInit) -> Result<(), Error> {
        let mut error: Option<Error> = None;
        let merge_error = |err: &mut Option<Error>, new: Error| {
            if err.is_none() {
                *err = Some(new);
            } else {
                err.as_mut().unwrap().merge(new);
            }
        };

        // Assets index
        Self::split_csv(&init.assets_bundle, |args| {
            let result = match args[0] {
                "TEXTURE" => {
                    self.load_texture(args)
                },
                "JSON" => {
                    self.load_json(init, args)
                },
                "CSV" => {
                    self.load_csv(init, args)
                },
                "SHADER" => Ok(()),
                _ => { Err(assets_err!("Unknown asset type {:?}", args[0])) }
            };

            if let Err(new_error) = result {
                merge_error(&mut error, new_error)
            }
        });

        if let Some(err) = error {
            return Err(err);
        }

        Ok(())
    }

    fn split_csv<CB: FnMut(&[&str])>(csv: &str, mut callback: CB) {
        let mut start = 0;
        let mut end = 0;
        let mut chars_iter = csv.chars();
        let mut args: [&str; 8] = [""; 8];
        while let Some(c) = chars_iter.next() {
            end += 1;
            if c == '\n' {
                let line = &csv[start..end];
                let mut args_count = 0;
                for substr in line.split(';') {
                    args[args_count] = substr;
                    args_count += 1;
                }

                if args_count > 1 {
                    callback(&args[0..(args_count-1)]);
                }

                start = end;
            }
        }
    }

    fn load_texture(&mut self, args: &[&str]) -> Result<(), Error> {
        let name = args.get(1)
            .map(|value| value.to_string() )
            .ok_or_else(|| assets_err!("Missing texture name") )?;

        let id = self.textures.len() as u32;
        self.textures.insert(name, Texture { id });

        Ok(())
    }

    fn load_json(&mut self, init: &crate::DemoGameInit, args: &[&str]) -> Result<(), Error> {
        let &json_name = args.get(2)
            .ok_or_else(|| assets_err!("Missing json name") )?;

        let &data_type = args.get(1)
            .ok_or_else(|| assets_err!("Missing json data type") )?;

        let json_string = init.text_assets.get(json_name)
            .ok_or_else(|| assets_err!("Failed to match json name to json data") )?;

        let json_value = serde_json::from_str::<serde_json::Value>(json_string.as_str())
            .map_err(|err| assets_err!("Failed to parse json data: {err}") )?;

        match data_type {
            "animation" => {
                self.animations.load_animation(json_name, json_value)?;
            },
            name => {
                return Err(assets_err!("Unknown json data type: {:?}", name));
            }
        };

        Ok(())
    }

    fn load_csv(&mut self, init: &crate::DemoGameInit, args: &[&str]) -> Result<(), Error> {
        let &csv_name = args.get(1)
            .ok_or_else(|| assets_err!("Missing csv name") )?;

        let csv_string = init.text_assets.get(csv_name)
            .ok_or_else(|| assets_err!("Failed to match csv name to csv data") )?;
        
        match csv_name {
            "terrain_sprites" => {
                dbg!("TODO load terrain sprites");
            },
            name => {
                return Err(assets_err!("Unknown csv: {:?}", name)); 
            }
        }

        Ok(())
    }
}

impl crate::store::SaveAndLoad for Assets {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_string_hashmap(&self.textures);
        writer.write(&self.animations);
        //writer.write(&self.terrain);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let mut assets = Assets::default();
        assets.textures = reader.read_string_hashmap();
        assets.animations = reader.read();
        //assets.terrain = reader.read();
        assets
    }
}
