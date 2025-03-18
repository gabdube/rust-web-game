//! Storage for the game data

use crate::shared::{Position, Size};
use crate::{assets, inputs, state, store, world};

#[derive(Copy, Clone, Default)]
pub struct DemoGameFlags {
    inner: u32,
}

macro_rules! flags {
    ($get:ident, $set:ident, $clear:ident, $value:expr) => {
        pub fn $set(&mut self) { self.inner |= $value; }
        pub fn $clear(&mut self) { self.inner &= !$value; }
        pub const fn $get(&self) -> bool { self.inner & $value > 0 }
    };
}

impl DemoGameFlags {
    const UPDATE_ANIMATIONS: u32 = 0b0001;   // Animations must be updated
    const SYNC_VIEW: u32         = 0b0010;   // World view offset must be synchronized with engine
    const SYNC_TERRAIN: u32      = 0b0100;   // Terrain data was changed and must be synchronized

    flags!(get_update_animations, set_update_animations, clear_update_animations, Self::UPDATE_ANIMATIONS);
    flags!(get_sync_view, set_sync_view, clear_sync_view, Self::SYNC_VIEW);
    flags!(get_sync_terrain, set_sync_terrain, clear_sync_terrain, Self::SYNC_TERRAIN);
}

#[derive(Default, Copy, Clone)]
pub struct DemoGameGlobalData {
    pub time: f64,
    pub last_animation_tick: f64,
    pub seed: u64,
    pub view_offset: Position<f32>,
    pub view_size: Size<f32>,
    pub frame_delta: f32,
    pub flags: DemoGameFlags,
}

/// The game data
pub struct DemoGameData {
    /// Values used everywhere in the app that do not fall into a specific category
    pub global: DemoGameGlobalData,
    /// State of the user input for the current frame
    pub inputs: inputs::InputState,
    /// Game static assets
    pub assets: assets::Assets,
    /// Game data
    pub world: world::World,
    /// Data unique to certain application state
    pub state: state::GameState,
}

impl DemoGameData {

    pub fn init_terrain(&mut self, width: u32, height: u32) {
        self.world.reset();
        self.world.init_terrain(width, height);
        self.global.flags.set_sync_terrain();
    }

}

impl Default for DemoGameData {
    fn default() -> Self {
        DemoGameData {
            global: DemoGameGlobalData::default(),
            inputs: inputs::InputState::default(),
            assets: assets::Assets::default(),
            world: world::World::default(),
            state: state::GameState::Startup,
        }
    }
}

impl store::SaveAndLoad for DemoGameGlobalData {
    fn save(&self, writer: &mut store::SaveFileWriter) {
        writer.write_f64(self.time);
        writer.write_f64(self.last_animation_tick);
        writer.write_u64(self.seed);
        writer.write(&self.view_offset);
        writer.write(&self.view_size);
        writer.write_f32(self.frame_delta);
        writer.write_u32(self.flags.inner);
    }

    fn load(reader: &mut store::SaveFileReader) -> Self {
        DemoGameGlobalData {
            time: reader.read_f64(),
            last_animation_tick: reader.read_f64(),
            seed: reader.read_u64(),
            view_offset: reader.read(),
            view_size: reader.read(),
            frame_delta: reader.read_f32(),
            flags: DemoGameFlags { inner: reader.read_u32() },
        }
    }
}

impl store::SaveAndLoad for DemoGameData {
    fn save(&self, writer: &mut store::SaveFileWriter) {
        writer.save(&self.assets);
        writer.save(&self.world);
        writer.save(&self.state);
        writer.save(&self.global);
    }

    fn load(reader: &mut store::SaveFileReader) -> Self {
        let assets = reader.load();
        let world = reader.load();
        let state = reader.load();
        let global = reader.load();

        DemoGameData {
            assets,
            world,
            state,
            global,
            inputs: Default::default(),
        }
    }
}

