/*!
    Packs objects sprites into a single atlas. Also generates a csv file with the name/uv offsets

    Call this script using `cargo run -p tools --release -- -c generate_gui_sprites`
*/

use png::OutputInfo;
use std::fs::File;
use crate::packing::{PackSprite, PackingState};
use crate::sprites::{SpriteData, SpriteInfo, PIXEL_SIZE};
use crate::shared::Rect;

const SRC_ROOT: &str = "build/assets/gui_assets/";
const DST_ROOT: &str = "build/assets/";
const DST_NAME_IMAGE: &str = "gui.png";
const DST_NAME_CSV: &str = "gui.csv";

const DST_WIDTH: usize = 768; // Manually tune this number to minimise wasted space

/// Sprites to pack in the objects atlas
static ASSETS: &[(&str, &str, SpriteInfo)] = &[
    ("info_panel", "info_panel.png", SpriteInfo::auto()),
    ("pawn_portrait", "units_portrait.png", SpriteInfo::sub(0, 0, 61, 61)),
    ("warrior_portrait", "units_portrait.png", SpriteInfo::sub(64, 0, 145, 95)),
    ("archer_portrait", "units_portrait.png", SpriteInfo::sub(146, 0, 212, 79)),

    ("gobindynamite_portrait", "units_portrait.png", SpriteInfo::sub(0, 95, 88, 165)),
    ("gobintorch_portrait", "units_portrait.png", SpriteInfo::sub(91, 95, 167, 177)),

    ("sheep_portrait", "units_portrait.png", SpriteInfo::sub(167, 124, 214, 169)),

    ("meat_icon", "meat.png", SpriteInfo::auto()),
    ("wood_icon", "wood.png", SpriteInfo::auto()),
    ("gold_icon", "gold.png", SpriteInfo::auto()),

    ("life_icon", "life2.png", SpriteInfo::auto()),
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
        eprintln!("Failed to write gui tilemap: {:?}", err);
        return;
    }

    println!("Gui tilemap written to \"{}{}\"", DST_ROOT, DST_NAME_CSV);
    println!("Gui image written to \"{}{}\"", DST_ROOT, DST_NAME_IMAGE);
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

    const PADDING: u32 = 1;

    let default_buffer_size = 2000;
    let mut csv_out = String::with_capacity(default_buffer_size);

    for (i, name) in state.sprite_names.iter().enumerate() {
        let [left, top, right, bottom] = state.sprites_dst[i].splat();
        let sprite_count = state.sprites_data[i].sprite_count();
        csv_out.push_str(&format!("{};{};{};{};{};{};\n", name, sprite_count, left+PADDING, top+PADDING, right-PADDING, bottom-PADDING));
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

