mod fonts;
pub use fonts::*;

mod animations;
pub use animations::AnimationBase;
use animations::AnimationsBundle;

mod terrain_tilemap;
pub use terrain_tilemap::{TerrainTilemap, TerrainCell};

mod decoration;
pub use decoration::*;

mod structures;
pub use structures::*;

mod resources;
pub use resources::*;

mod gui;
pub use gui::GuiBundle;

use fnv::FnvHashMap;

use crate::error::Error;
use crate::shared::AABB;
use crate::world::WorldObjectType;
use crate::{DemoGame, DemoGameInit};

#[derive(Copy, Clone)]
pub struct Texture {
    // The unique ID of the texture. It is also the index of the texture in the engine texture array
    pub id: u32,
}

#[derive(Default)]
pub struct Fonts {
    // Assets are shared with the gui system
    pub roboto: FontAsset,
}

#[derive(Default)]
pub struct Assets {
    pub textures: FnvHashMap<String, Texture>,
    pub fonts: Fonts,
    pub terrain: TerrainTilemap,
    pub gui: GuiBundle,
    pub decorations: DecorationBundle,
    pub structures: StructuresBundle,
    pub resources: ResourcesBundle,
    pub animations: AnimationsBundle,
}

impl Assets {

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
                self.animations.load_animations(csv_string)?;
            },
            "gui" => {
                self.gui.load(csv_string);
            },
            name => {
                warn!("Unknown csv: {:?}", name);
            }
        }

        Ok(())
    }

    fn load_font(&mut self, init: &crate::DemoGameInit, args: &[&str]) -> Result<(), Error> {
        let font_name = args.get(1)
            .map(|value| value.to_string() )
            .ok_or_else(|| assets_err!("Missing font name") )?;

        let font_atlas_data = init.font_assets.get(&font_name)
            .ok_or_else(|| assets_err!("Failed to match font name to font data") )?;

        let texture_id = self.textures.len() as u32;
        let font = FontAsset::from_bytes(texture_id, font_atlas_data)?;

        match font_name.as_str() {
            "roboto" => { self.fonts.roboto = font; },
            name => { warn!("Unknown font: {:?}", name); }
        };

        self.textures.insert(font_name, Texture { id: texture_id });

        Ok(())
    }

    pub fn object_gui_image(&self, object_type: WorldObjectType) -> AABB {
        let gui = &self.gui;
        match object_type {
            WorldObjectType::Pawn => gui.pawn_portrait,
            WorldObjectType::Warrior => gui.warrior_portrait,
            WorldObjectType::Archer => gui.archer_portrait,
            WorldObjectType::TorchGoblin => gui.goblin_torch_portrait,
            WorldObjectType::DynamiteGoblin => gui.goblin_dynamite_portrait,
            WorldObjectType::Sheep => gui.sheep_portrait,
            _ => AABB::default()
        }
    }
}

pub fn init_assets(game: &mut DemoGame, init: &DemoGameInit) -> Result<(), Error> {
    import_assets_index(game, &init)?;
    init_world_assets(game)?;
    Ok(())
}

fn import_assets_index(game: &mut DemoGame, init: &DemoGameInit) -> Result<(), Error> {
    let mut error: Option<Error> = None;
    let assets = &mut game.data.assets;

    // Assets index
    crate::shared::split_csv::<5, _>(&init.assets_bundle, |args| {
        let result = match args[0] {
            "TEXTURE" => {
                assets.load_texture(args)
            },
            "CSV" => {
                assets.load_csv(init, args)
            },
            "FONT" => {
                assets.load_font(init, args)
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

fn init_world_assets(game: &mut DemoGame) -> Result<(), Error> {
    let world = &mut game.data.world;
    let assets = &game.data.assets;

    world.units_texture = assets.textures.get("units").copied()
        .ok_or_else(|| assets_err!("units texture missing") )?;

    world.static_resources_texture = assets.textures.get("static_resources").copied()
        .ok_or_else(|| assets_err!("static_resources texture missing") )?;

    Ok(())
}

//
// Other Impls
//

impl crate::store::SaveAndLoad for Assets {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_string_hashmap(&self.textures);
        writer.save(&self.terrain);
        writer.write(&self.decorations);
        writer.write(&self.structures);
        writer.write(&self.resources);
        writer.write(&self.animations);
        writer.write(&self.gui);
        writer.save(&self.fonts);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let mut assets = Assets::default();
        assets.textures = reader.read_string_hashmap();
        assets.terrain = reader.load();
        assets.decorations = reader.read();
        assets.structures = reader.read();
        assets.resources = reader.read();
        assets.animations = reader.read();
        assets.gui = reader.read();
        assets.fonts = reader.load();
        assets
    }
}

impl crate::store::SaveAndLoad for Fonts {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.save(&self.roboto);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        Fonts { 
            roboto: reader.load(),
        }
    }

}
