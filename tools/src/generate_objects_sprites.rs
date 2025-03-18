/*!
    Packs objects sprites into a single atlas. Also generates a csv file with the name/uv offsets

    Call this script using `cargo run -p tools --release -- -c generate_objects_sprites`
*/
use png::OutputInfo;
use std::fs::File;
use crate::packing::{PackSprite, PackingState};
use crate::sprites::{SpriteData, SpriteInfo, PIXEL_SIZE};
use crate::shared::{Rect, rect, size};

const SRC_ROOT: &str = "build/assets/tiny_sword/";
const DST_ROOT: &str = "build/assets/";
const DST_NAME_IMAGE: &str = "static_resources.png";
const DST_NAME_CSV: &str = "static_resources.csv";

const DST_WIDTH: usize = 1054; // Manually tune this number to minimise wasted space

/// Sprites to pack in the objects atlas
static ASSETS: &[(&str, &str, SpriteInfo)] = &[
    // Deco
    ("shroom_small", "Deco/01.png", SpriteInfo::auto()),
    ("shroom_medium", "Deco/02.png", SpriteInfo::auto()),
    ("shroom_big", "Deco/03.png", SpriteInfo::auto()),
    ("rock_small", "Deco/04.png", SpriteInfo::auto()),
    ("rock_medium", "Deco/05.png", SpriteInfo::auto()),
    ("rock_big", "Deco/06.png", SpriteInfo::auto()),
    ("bush_small", "Deco/07.png", SpriteInfo::auto()),
    ("bush_medium", "Deco/08.png", SpriteInfo::auto()),
    ("bush_big", "Deco/09.png", SpriteInfo::auto()),
    ("plant_small", "Deco/10.png", SpriteInfo::auto()),
    ("plant_medium", "Deco/11.png", SpriteInfo::auto()),
    ("pumpkin_small", "Deco/12.png", SpriteInfo::auto()),
    ("pumpkin_medium", "Deco/13.png", SpriteInfo::auto()),
    ("bone_1", "Deco/14.png", SpriteInfo::auto()),
    ("bone_2", "Deco/15.png", SpriteInfo::auto()),
    ("sign_1", "Deco/16.png", SpriteInfo::auto()),
    ("sign_2", "Deco/17.png", SpriteInfo::auto()),
    ("sign_3", "Deco/18.png", SpriteInfo::auto()),

    // Goblin
    ("goblin_house", "Factions/Goblins/Buildings/Wood_House/Goblin_House.png", SpriteInfo::auto()),
    ("goblin_house_destroyed", "Factions/Goblins/Buildings/Wood_House/Goblin_House_Destroyed.png", SpriteInfo::auto()),
    ("explosive_barrel", "Factions/Goblins/Troops/Barrel/Red/Barrel_Red.png", SpriteInfo::sub(39, 29, 89, 99)),
    ("explosive_barrel_triggered", "Factions/Goblins/Troops/Barrel/Red/Barrel_Red.png", SpriteInfo::animated(rect(0, 640, 384, 768), size(128, 128))),
    ("tnt", "Factions/Goblins/Troops/TNT/Dynamite/Dynamite.png", SpriteInfo::animated(rect(0, 0, 384, 64), size(64, 64))),

    // Knights
    ("knight_castle", "Factions/Knights/Buildings/Castle/Castle_Blue.png", SpriteInfo::auto()),
    ("knight_castle_construction", "Factions/Knights/Buildings/Castle/Castle_Construction.png", SpriteInfo::auto()),
    ("knight_castle_destroyed", "Factions/Knights/Buildings/Castle/Castle_Destroyed.png", SpriteInfo::auto()),
    ("knight_house", "Factions/Knights/Buildings/House/House_Blue.png", SpriteInfo::auto()),
    ("knight_house_construction", "Factions/Knights/Buildings/House/House_Construction.png", SpriteInfo::auto()),
    ("knight_house_destroyed", "Factions/Knights/Buildings/House/House_Destroyed.png", SpriteInfo::auto()),
    ("knight_tower", "Factions/Knights/Buildings/Tower/Tower_Blue.png", SpriteInfo::auto()),
    ("knight_tower_construction", "Factions/Knights/Buildings/Tower/Tower_Construction.png", SpriteInfo::auto()),
    ("knight_tower_destroyed", "Factions/Knights/Buildings/Tower/Tower_Destroyed.png", SpriteInfo::auto()),
    
    ("arrow", "Factions/Knights/Troops/Archer/Arrow/Arrow.png", SpriteInfo::sub(0, 0, 64, 64)),
    ("arrow_stuck", "Factions/Knights/Troops/Archer/Arrow/Arrow.png", SpriteInfo::sub(0, 64, 64, 128)),

    // Resources
    ("gold_mine", "Resources/Gold Mine/GoldMine_Active.png", SpriteInfo::SimpleNoTrimming),
    ("gold_mine_destroyed", "Resources/Gold Mine/GoldMine_Destroyed.png", SpriteInfo::SimpleNoTrimming),
    ("gold_mine_inactive", "Resources/Gold Mine/GoldMine_Inactive.png", SpriteInfo::SimpleNoTrimming),
    ("gold_noshadow", "Resources/Resources/G_Idle_(NoShadow).png", SpriteInfo::auto()),
    ("gold", "Resources/Resources/G_Idle.png", SpriteInfo::auto()),
    ("meat_noshadow", "Resources/Resources/M_Idle_(NoShadow).png", SpriteInfo::auto()),
    ("meat", "Resources/Resources/M_Idle.png", SpriteInfo::auto()),
    ("wood_noshadow", "Resources/Resources/W_Idle_(NoShadow).png", SpriteInfo::auto()),
    ("wood", "Resources/Resources/W_Idle.png", SpriteInfo::auto()),
    ("tree_stump", "Resources/Trees/Tree.png", SpriteInfo::sub(79, 530, 115, 560)),
    ("tree_idle", "Resources/Trees/Tree.png", SpriteInfo::animated(rect(0, 0, 768, 192), size(192, 192))),
    ("tree_cut", "Resources/Trees/Tree.png", SpriteInfo::animated(rect(0, 192, 384, 384), size(192, 192))),
    ("gold_spawn", "Resources/Resources/G_Spawn.png", SpriteInfo::animated(rect(0, 0, 896, 128), size(128, 128))),
    ("meat_spawn", "Resources/Resources/M_Spawn.png", SpriteInfo::animated(rect(0, 0, 896, 128), size(128, 128))),
    ("wood_spawn", "Resources/Resources/W_Spawn.png", SpriteInfo::animated(rect(0, 0, 896, 128), size(128, 128))),

    // Effects
    ("fire", "Effects/Fire/Fire.png", SpriteInfo::animated(rect(0, 0, 896, 128), size(128, 128))),
    ("explosion", "Effects/Explosion/Explosions.png", SpriteInfo::animated(rect(0, 0, 1728, 192), size(192, 192))),
];

struct AssetsState {
    sprite_names: Vec<&'static str>,
    sprites_data: Vec<SpriteData>,
    sprites_dst: Vec<Rect>,

    output_image_bytes: Vec<u8>,
    output_image_info: OutputInfo,
}

//
// Processing sprites
//

pub fn generate_sprites() {
    let total_sprites_count = ASSETS.len();

    let mut state = AssetsState {
        sprite_names: Vec::with_capacity(total_sprites_count),
        sprites_data: Vec::with_capacity(total_sprites_count),
        sprites_dst: Vec::with_capacity(total_sprites_count),

        output_image_bytes: Vec::new(),
        output_image_info: OutputInfo {
            width: DST_WIDTH as u32,
            height: 0,
            bit_depth: png::BitDepth::Eight,
            color_type: png::ColorType::Rgba,
            line_size: DST_WIDTH * PIXEL_SIZE,
        }
    };

    load_sprite_sources(&mut state);
    generate_tilemap(&mut state);
    
    if let Err(err) = write_tilemap(&mut state) {
        eprintln!("Failed to write tilemap: {:?}", err);
        return;
    }

    println!("Static resources tilemap written to \"{}{}\"", DST_ROOT, DST_NAME_CSV);
    println!("Static resources image written to \"{}{}\"", DST_ROOT, DST_NAME_IMAGE);
}

//
// Loading images
//

fn load_sprite_sources(state: &mut AssetsState) {
    for (name, path, sprite_info) in ASSETS.iter() {
        state.sprite_names.push(name);
        state.sprites_dst.push(Rect::default());

        let path = format!("{SRC_ROOT}{path}");
        state.sprites_data.push(SpriteData::load (&path, &sprite_info))
    }
}


// 
// Generating tilemaps
//

fn check_min_dimensions(state: &AssetsState) {
    let min_width = state.sprites_data.iter().map(|v| v.size.width ).max().unwrap_or(0) as usize;
    if min_width > DST_WIDTH {
        panic!("MIN_WIDTH ({DST_WIDTH}) must be at least as large as the longest sprite {min_width}");
    }
}

/// Maps the state data into a format that can be processed by `pack_sprites` and sort them by size
fn generate_pack_sprites(state: &mut AssetsState) -> PackingState {
    let sprites = state.sprites_data.iter().enumerate()
        .map(|(index, sprite)| 
            PackSprite { 
                index: index as u32,
                size: sprite.size,
                rect: Default::default()
            }
        )
        .collect();

    PackingState::new(sprites)
}

/// Transfer data from `PackState` to `AssetState`
fn transfer_data(state: &mut AssetsState, pack: &PackingState) {
    for sprite in pack.sprites() {
        state.sprites_dst[sprite.index as usize] = sprite.rect;
    }

    state.output_image_info.height = pack.size().height;
}

/// Allocate space for the tilemap and copy the sprites from `AssetsState` into it
fn copy_sprites(state: &mut AssetsState) {
    fn copy_sprite(
        dst: &mut [u8], dst_x: usize, dst_y: usize, dst_stride: usize,
        src: &[u8], src_stride: usize, height: usize
    ){
        for line in 0..height {
            let src_offset = line * src_stride;
            let dst_offset = ((line+dst_y) * dst_stride) + (dst_x * PIXEL_SIZE);
            unsafe {
                ::std::ptr::copy_nonoverlapping(
                    src.as_ptr().add(src_offset),
                    dst.as_mut_ptr().add(dst_offset),
                    src_stride
                );
            }
        }
    }

    let dst_stride = state.output_image_info.width as usize * PIXEL_SIZE;
    let total_image_size = state.output_image_info.height as usize * dst_stride;
    let mut dst_bytes = vec![0; total_image_size];

    for (i, sprite) in state.sprites_data.iter().enumerate() {
        let dst_rect = state.sprites_dst[i];
        let dst_x = dst_rect.left as usize;
        let dst_y = dst_rect.top as usize;
        let height = sprite.size.height as usize;
        let src_stride = sprite.line_size();
        copy_sprite(
            &mut dst_bytes, dst_x, dst_y, dst_stride,
            &sprite.pixels, src_stride, height,
        );
    }

    state.output_image_bytes = dst_bytes;
}

fn generate_tilemap(state: &mut AssetsState) {
    check_min_dimensions(state);

    let mut pack_state = generate_pack_sprites(state);
    pack_state.compute(DST_WIDTH);

    transfer_data(state, &pack_state);
    copy_sprites(state);
}

//
// Dst copy & csv generation
//

fn write_tilemap_image(state: &mut AssetsState) -> Result<(), Box<dyn ::std::error::Error>> {
    use std::io::BufWriter;

    let out_path = format!("{DST_ROOT}{DST_NAME_IMAGE}");
    let file = File::create(&out_path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, state.output_image_info.width, state.output_image_info.height);
    encoder.set_compression(png::Compression::Best);
    encoder.set_color(state.output_image_info.color_type);
    encoder.set_depth(state.output_image_info.bit_depth);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455));
    let source_chromaticities = png::SourceChromaticities::new(
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000)
    );
    encoder.set_source_chromaticities(source_chromaticities);
    let mut writer = encoder.write_header()?;

    writer.write_image_data(&state.output_image_bytes)?;

    Ok(())
}

fn write_tilemap_csv(state: &mut AssetsState) -> Result<(), Box<dyn ::std::error::Error>> {
    use std::io::Write;

    let default_buffer_size = 2000;
    let mut csv_out = String::with_capacity(default_buffer_size);

    for (i, name) in state.sprite_names.iter().enumerate() {
        let [left, top, right, bottom] = state.sprites_dst[i].splat();
        let sprite_count = state.sprites_data[i].sprite_count();
        csv_out.push_str(&format!("{};{};{};{};{};{};\n", name, sprite_count, left, top, right, bottom));
    }

    let out_path = format!("{DST_ROOT}{DST_NAME_CSV}");
    let mut file = File::create(&out_path)?;
    file.write(csv_out.as_bytes())?;

    Ok(())
}

fn write_tilemap(state: &mut AssetsState) -> Result<(), Box<dyn ::std::error::Error>> {
    write_tilemap_image(state)?;
    write_tilemap_csv(state)?;
    Ok(())
}
