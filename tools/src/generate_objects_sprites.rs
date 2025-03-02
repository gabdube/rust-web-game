/*!
    Packs objects sprites into a single atlas. Also generates a csv file with the name/uv offsets

    Call this script using `cargo run -p tools --release -- -c generate_objects_sprites`
*/
use png::OutputInfo;
use std::{cmp::Ordering, fs::File, u32};
use crate::sprites::AnimationInfo;
use crate::shared::{Rect, Size, rect, size};

const SRC_ROOT: &str = "build/assets/tiny_sword/";
const DST_ROOT: &str = "build/assets/";
const DST_NAME_IMAGE: &str = "static_resources.png";
const DST_NAME_CSV: &str = "static_resources.csv";

const DST_WIDTH: usize = 890; // Manually tune this number to minimise wasted space
const PIXEL_SIZE: usize = 4; // Size of rgba u8

/// Sprites that are single images. Blank space around the sprites will be cropped
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

    // Knights
    ("knight_castle", "Factions/Knights/Buildings/Castle/Castle_Blue.png"),
    ("knight_castle_construction", "Factions/Knights/Buildings/Castle/Castle_Construction.png"),
    ("knight_castle_destroyed", "Factions/Knights/Buildings/Castle/Castle_Destroyed.png"),
    ("knight_house", "Factions/Knights/Buildings/House/House_Blue.png"),
    ("knight_house_construction", "Factions/Knights/Buildings/House/House_Construction.png"),
    ("knight_house_destroyed", "Factions/Knights/Buildings/House/House_Destroyed.png"),
    ("knight_tower", "Factions/Knights/Buildings/Tower/Tower_Blue.png"),
    ("knight_tower_construction", "Factions/Knights/Buildings/Tower/Tower_Construction.png"),
    ("knight_tower_destroyed", "Factions/Knights/Buildings/Tower/Tower_Destroyed.png"),

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
    ("explosive_barrel", "Factions/Goblins/Troops/Barrel/Red/Barrel_Red.png", rect(39, 29, 89, 99)),
    ("tree_stump", "Resources/Trees/Tree.png", rect(79, 530, 115, 560)),
];

/// Animated sprites
const SRC_ASSET_MAP_3: &[(&str, &str, AnimationInfo)] = &[
    ("tree_idle", "Resources/Trees/Tree.png", AnimationInfo::new(rect(0, 0, 768, 192), size(192, 192))),
    ("tree_cut", "Resources/Trees/Tree.png", AnimationInfo::new(rect(0, 192, 384, 384), size(192, 192))),
    ("gold_spawn", "Resources/Resources/G_Spawn.png", AnimationInfo::new(rect(0, 0, 896, 128), size(128, 128))),
    ("meat_spawn", "Resources/Resources/M_Spawn.png", AnimationInfo::new(rect(0, 0, 896, 128), size(128, 128))),
    ("wood_spawn", "Resources/Resources/W_Spawn.png", AnimationInfo::new(rect(0, 0, 896, 128), size(128, 128))),
];


struct Sprite {
    pub bytes: Vec<u8>,
    pub src_image_info: OutputInfo,
    pub name: &'static str,
    pub dst_rect: Rect,
}

struct AssetsState {
    sprites: Vec<Sprite>,
    output_image_bytes: Vec<u8>,
    output_image_info: OutputInfo,
}

//
// Processing sprites
//

pub fn generate_sprites() {
    let mut state = AssetsState {
        sprites: Vec::with_capacity(SRC_ASSET_MAP.len() + SRC_ASSET_MAP_2.len()),
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

fn load_image_base(path: &str) -> (Vec<u8>, OutputInfo) {
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

fn copy_sprite_bytes(src_image_info: &OutputInfo, src_rect: &Rect, src_bytes: &[u8]) -> Vec<u8> {
    let width = src_rect.width() as usize;
    let height = src_rect.height() as usize;
    let mut dst_bytes = Vec::with_capacity(width * height * PIXEL_SIZE);

    let top = src_rect.top as usize;
    let bottom = src_rect.bottom as usize;
    let left = src_rect.left as usize;
    let dst_line_size = width * PIXEL_SIZE;

    for i in top..bottom {
        let bytes_start = (i * src_image_info.line_size) + (left * PIXEL_SIZE);
        dst_bytes.extend_from_slice(&src_bytes[bytes_start..bytes_start+dst_line_size]);
    }

    dst_bytes
}

fn load_simple_image(path: &str) -> (Vec<u8>, OutputInfo) {
    let (bytes_all, mut src_image_info) = load_image_base(path);

    // Strip the extra whitespace around the sprites
    let mut rect = Rect::default();
    rect.left = u32::MAX;
    rect.top = u32::MAX;

    for y in 0..src_image_info.height {
        for x in 0..src_image_info.width {
            let [x2, y2] = [x as usize, y as usize];
            let pixel_offset = (y2 * src_image_info.line_size) + (x2 * PIXEL_SIZE);
            let a: u8 = bytes_all[pixel_offset + 3];
            if a != 0 {
                rect.left = u32::min(rect.left, x);
                rect.right = u32::max(rect.right, x);
                rect.top = u32::min(rect.top, y);
                rect.bottom = u32::max(rect.bottom, y);
            }
        }
    }

    if rect.width() == 0 || rect.height() == 0 {
        panic!("Failed to read {:?} pixels", path);
    }
    
    let bytes = copy_sprite_bytes(&src_image_info, &rect, &bytes_all);

    src_image_info.width = rect.width();
    src_image_info.height = rect.height();
    src_image_info.line_size = rect.width() as usize * PIXEL_SIZE;

    (bytes, src_image_info)
}

fn load_sub_image(path: &str, rect: &Rect) -> (Vec<u8>, OutputInfo) {
    let (bytes_all, mut src_image_info) = load_image_base(path);
    let bytes = copy_sprite_bytes(&src_image_info, &rect, &bytes_all);
    src_image_info.width = rect.width() as u32;
    src_image_info.height = rect.height() as u32;
    src_image_info.line_size = rect.width() as usize * PIXEL_SIZE;

    (bytes, src_image_info)
}

fn load_sprite_sources(state: &mut AssetsState) {
    for (name, path) in SRC_ASSET_MAP.iter() {
        let (bytes, src_image_info) = load_simple_image(path);
        let sprite = Sprite {
            bytes,
            src_image_info,
            name: *name,
            dst_rect: Rect::default(),
        };

        state.sprites.push(sprite);
    }

    for (name, path, src_rect) in SRC_ASSET_MAP_2.iter() {
        let (bytes, src_image_info) = load_sub_image(path, src_rect);
        let sprite = Sprite {
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

struct PackingState<'a> {
    sprites: &'a mut [Sprite],
    index_size_processed: &'a mut [(usize, Size, bool)],
    area: Rect,
}

impl<'a> PackingState<'a> {
    fn from_rect(sprites: &'a mut [Sprite], processed: &'a mut [(usize, Size, bool)], left: u32, top: u32, width: u32, height: u32) -> Self {
        PackingState {
            sprites,
            index_size_processed: processed,
            area: Rect { left, top, right: left+width, bottom: top+height }
        }
    }

    fn sub_section<'b>(&'b mut self, width: u32, height: u32) -> PackingState<'b> {
        PackingState {
            sprites: self.sprites,
            index_size_processed: self.index_size_processed,
            area: Rect {
                left: self.area.left,
                top: self.area.top + height,
                right: self.area.left + width,
                bottom: self.area.bottom,
            }
        }
    }

    fn store_next_sprite(&mut self) -> Option<Size> {
        let (index, size) = self.index_size_processed.iter_mut()
            .find(|(_, size, processed)| {
                *processed == false && self.area.fits(size.width, size.height)
            })
            .map(|(index, size, processed)| {
                *processed = true;
                (*index, *size)
            })?;

        self.sprites[index].dst_rect = Rect { left: self.area.left, top: self.area.top, right: self.area.left + size.width, bottom: self.area.top + size.height };

        Some(size)
    }

    fn has_remaining_items(&self) -> bool {
        self.index_size_processed.iter().any(|(_, _, processed)| *processed )
    }
}

/// Order sprites by height, then by width
fn order_sprites(state: &mut AssetsState) {
    fn sort_sprites(i1: &Sprite, i2: &Sprite) -> Ordering {
        if i2.src_image_info.height < i1.src_image_info.height {
            return Ordering::Less;
        } else if i2.src_image_info.height > i1.src_image_info.height {
            return Ordering::Greater;
        } else {
            return i2.src_image_info.width.cmp(&i1.src_image_info.width);
        }
    }

    state.sprites.sort_unstable_by(sort_sprites);
}

fn pack_row(state: &mut PackingState) -> bool {
    loop {
        let size = match state.store_next_sprite() {
            Some(value) => value,
            None => { return state.has_remaining_items(); }
        };

        if size.height <= state.area.height() {
            let mut state = state.sub_section(size.width, size.height);
            if !pack_row(&mut state) {
                return false;
            }
        }

        state.area.left += size.width;
    }
}

fn pack_sprites(state: &mut AssetsState) {
    let mut sprites_sizes: Vec<(usize, Size, bool)> = state.sprites.iter().enumerate()
        .map(|(i, sprite)| (i, size(sprite.src_image_info.width, sprite.src_image_info.height), false))
        .collect();

    let mut top = 0;

    loop {
        let size = match sprites_sizes.iter().find(|(_, _, processed)| *processed == false ).map(|(_, size, _)| *size ) {
            Some(size) => size,
            None => { break; }
        };

        let mut pack_state = PackingState::from_rect(&mut state.sprites, &mut sprites_sizes, 0, top, DST_WIDTH as u32, size.height);
        if !pack_row(&mut pack_state) {
            break;
        }

        top += size.height;
    }

    state.output_image_info.height = top;
}

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

    for sprite in state.sprites.iter() {
        if sprite.dst_rect.width() == 0 {
            continue;
        }

        let dst_x = sprite.dst_rect.left as usize;
        let dst_y = sprite.dst_rect.top as usize;
        let src_stride = sprite.src_image_info.line_size;
        let height = sprite.src_image_info.height as usize;
        copy_sprite(
            &mut dst_bytes, dst_x, dst_y, dst_stride,
            &sprite.bytes, src_stride, height,
        );
    }

    state.output_image_bytes = dst_bytes;
}

fn generate_tilemap(state: &mut AssetsState) {
    order_sprites(state);
    pack_sprites(state);
    copy_sprites(state);

    // for sprite in state.sprites.iter() {
    //     println!("{:?} {:?}", sprite.name, sprite.dst_rect);
    // }
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

    for sprite in state.sprites.iter() {
        let [left, top, right, bottom] = sprite.dst_rect.splat();
        csv_out.push_str(&format!("{};{};{};{};{};\n", sprite.name, left, top, right, bottom));
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
