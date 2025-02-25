/*!
    Packs static (non animated) sprites into a single atlas. Also generates a csv file with the name/uv offsets

    Call this script using `cargo run -p tools --release -- -c generate_static_sprites`
*/
use png::OutputInfo;
use std::fs::File;
use crate::shared::Rect;

const SRC_ROOT: &str = "build/assets/tiny_sword/";
const DST_ROOT: &str = "build/assets/";
const DST_NAME_IMAGE: &str = "static_resources.png";
const DST_NAME_CSV: &str = "static_resources.csv";
const DST_WIDTH: usize = 2048;

const SRC_ASSET_MAP: &[(&str, &str)] = &[
    // Deco
    ("shroom_small", "Deco/01.png"),
    ("shroom_medium", "Deco/02.png"),
    ("shroom_big", "Deco/03.png"),
    ("rock_small", "Deco/04.png"),
    ("rock_medium", "Deco/05.png"),
    ("rock_big", "Deco/06.png"),
    ("bush_small", "Deco/07.png"),
    ("bush_medium", "Deco/08.png"),
    ("bush_big", "Deco/09.png"),
    ("plant_small", "Deco/10.png"),
    ("plant_medium", "Deco/11.png"),
    ("pumpkin_small", "Deco/12.png"),
    ("pumpkin_medium", "Deco/13.png"),
    ("bone_1", "Deco/14.png"),
    ("bone_2", "Deco/15.png"),
    ("sign_1", "Deco/16.png"),
    ("sign_2", "Deco/17.png"),
    ("sign_3", "Deco/18.png"),

    // Goblin
    ("goblin_house", "Factions/Goblins/Buildings/Wood_House/Goblin_House.png"),
    ("goblin_house_destroyed", "Factions/Goblins/Buildings/Wood_House/Goblin_House_Destroyed.png"),
    ("goblin_tower", "Factions/Goblins/Buildings/Wood_Tower/Wood_Tower_Red.png"),
    ("goblin_tower_destroyed", "Factions/Goblins/Buildings/Wood_Tower/Wood_Tower_Destroyed.png"),

    // Knights
    ("knights_castle", "Factions/Knights/Buildings/Castle/Castle_Blue.png"),
    ("knights_castle_construction", "Factions/Knights/Buildings/Castle/Castle_Construction.png"),
    ("knights_castle_destroyed", "Factions/Knights/Buildings/Castle/Castle_Destroyed.png"),
    ("knights_house", "Factions/Knights/Buildings/House/House_Blue.png"),
    ("knights_house_construction", "Factions/Knights/Buildings/House/House_Construction.png"),
    ("knights_house_destroyed", "Factions/Knights/Buildings/House/House_Destroyed.png"),
    ("knights_tower", "Factions/Knights/Buildings/Tower/Tower_Blue.png"),
    ("knights_tower_construction", "Factions/Knights/Buildings/Tower/Tower_Construction.png"),
    ("knights_tower_destroyed", "Factions/Knights/Buildings/Tower/Tower_Destroyed.png"),

    // Resources
    ("gold_mine", "Resources/Gold Mine/GoldMine_Active.png"),
    ("gold_mine_destroyed", "Resources/Gold Mine/GoldMine_Destroyed.png"),
    ("gold_mine_inactive", "Resources/Gold Mine/GoldMine_Inactive.png"),
    ("gold_noshadow", "Resources/Resources/G_Idle_(NoShadow).png"),
    ("gold", "Resources/Resources/G_Idle.png"),
    ("meat_noshadow", "Resources/Resources/M_Idle_(NoShadow).png"),
    ("meat", "Resources/Resources/M_Idle.png"),
    ("wood_noshadow", "Resources/Resources/W_Idle_(NoShadow).png"),
    ("wood", "Resources/Resources/W_Idle.png"),
];

/// Sprites that are a subset of a bigger image
const SRC_ASSET_MAP_2: &[(&str, &str, Rect)] = &[
    ("explosive_barrel", "Factions/Goblins/Troops/Barrel/Red/Barrel_Red.png", Rect { left: 37, top: 27, right: 92, bottom: 99 }),
    ("tree_stump", "Resources/Trees/Tree.png", Rect { left: 78, top: 525, right: 118, bottom: 561 }),
];

struct Sprite {
    pub bytes: Vec<u8>,
    pub src_image_info: OutputInfo,
    pub name: &'static str,
    pub dst_rect: Rect,
}

struct AssetsState {
    sprites: Vec<Sprite>
}

//
// Processing sprites
//



pub fn generate_sprites() {
    let mut state = AssetsState {
        sprites: Vec::with_capacity(SRC_ASSET_MAP.len() + SRC_ASSET_MAP_2.len())
    };

    load_sprite_sources(&mut state);
}

//
// Loading images
//

fn load_simple_image(path: &str) -> (Vec<u8>, OutputInfo) {
    let final_path = format!("{SRC_ROOT}{path}");
    let file = match File::open(&final_path) {
        Ok(f) => f,
        Err(e) => {
            panic!("Failed to open {final_path:?}: {e:?}");
        }
    };

    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().unwrap();

    let mut bytes = vec![0; reader.output_buffer_size()];
    let image_info = reader.next_frame(&mut bytes).unwrap();

    match (image_info.bit_depth, image_info.color_type) {
        (png::BitDepth::Eight, png::ColorType::Rgba) => { /* OK */ },
        combined => unimplemented!("batching sprites for {:?} is not implemented", combined)
    }

    (bytes, image_info)
}

fn load_sub_image(path: &str, rect: &Rect) -> (Vec<u8>, OutputInfo) {
    const PIXEL_SIZE: usize = 4;
    let (bytes_all, mut src_image_info) = load_simple_image(path);

    let width = rect.width() as usize;
    let height = rect.height() as usize;
    let mut bytes = Vec::with_capacity(width * height * PIXEL_SIZE);

    // Copies the subset of the image delimited by `rect` into its own buffer
    let top = rect.top as usize;
    let bottom = rect.bottom as usize;
    let left = rect.left as usize;
    let dst_line_size = width * PIXEL_SIZE;

    for i in top..bottom {
        let bytes_start = (i * src_image_info.line_size) + (left * PIXEL_SIZE);
        if bytes_start+dst_line_size > bytes_all.len() {
            println!("{} {:?}", i, path);
        };
        bytes.extend_from_slice(&bytes_all[bytes_start..bytes_start+dst_line_size]);
    }

    src_image_info.width = width as u32;
    src_image_info.height = height as u32;
    src_image_info.line_size = dst_line_size;

    (bytes, src_image_info)
}

fn load_sprite_sources(state: &mut AssetsState) {
    for (name, path) in SRC_ASSET_MAP.iter() {
        let (bytes, src_image_info) = load_simple_image(path);

        let mut sprite = Sprite {
            bytes,
            src_image_info,
            name: *name,
            dst_rect: Rect::default(),
        };

        state.sprites.push(sprite);
    }

    for (name, path, src_rect) in SRC_ASSET_MAP_2.iter() {
        let (bytes, src_image_info) = load_sub_image(path, src_rect);
        let mut sprite = Sprite {
            bytes,
            src_image_info,
            name: *name,
            dst_rect: Rect::default(),
        };

        state.sprites.push(sprite);
    }
}


//
// Generating tilemaps
//

//
// Writing images & json
//

