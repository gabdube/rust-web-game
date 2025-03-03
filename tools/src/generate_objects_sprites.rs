/*!
    Packs objects sprites into a single atlas. Also generates a csv file with the name/uv offsets

    Call this script using `cargo run -p tools --release -- -c generate_objects_sprites`
*/
use png::OutputInfo;
use std::{cmp::Ordering, fs::File, u32};
use crate::sprites::{SpriteData, SpriteInfo, PIXEL_SIZE};
use crate::shared::{Rect, Size, rect, size};

const SRC_ROOT: &str = "build/assets/tiny_sword/";
const DST_ROOT: &str = "build/assets/";
const DST_NAME_IMAGE: &str = "static_resources.png";
const DST_NAME_CSV: &str = "static_resources.csv";

const DST_WIDTH: usize = 890; // Manually tune this number to minimise wasted space

/// Sprites that are single images. Blank space around the sprites will be cropped
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

    // Resources
    ("gold_mine", "Resources/Gold Mine/GoldMine_Active.png", SpriteInfo::auto()),
    ("gold_mine_destroyed", "Resources/Gold Mine/GoldMine_Destroyed.png", SpriteInfo::auto()),
    ("gold_mine_inactive", "Resources/Gold Mine/GoldMine_Inactive.png", SpriteInfo::auto()),
    ("gold_noshadow", "Resources/Resources/G_Idle_(NoShadow).png", SpriteInfo::auto()),
    ("gold", "Resources/Resources/G_Idle.png", SpriteInfo::auto()),
    ("meat_noshadow", "Resources/Resources/M_Idle_(NoShadow).png", SpriteInfo::auto()),
    ("meat", "Resources/Resources/M_Idle.png", SpriteInfo::auto()),
    ("wood_noshadow", "Resources/Resources/W_Idle_(NoShadow).png", SpriteInfo::auto()),
    ("wood", "Resources/Resources/W_Idle.png", SpriteInfo::auto()),
    ("tree_stump", "Resources/Trees/Tree.png", SpriteInfo::sub(79, 530, 115, 560)),
    //("tree_idle", "Resources/Trees/Tree.png", SpriteInfo::animated(rect(0, 0, 768, 192), size(192, 192))),
    //("tree_cut", "Resources/Trees/Tree.png", SpriteInfo::animated(rect(0, 192, 384, 384), size(192, 192))),
    //("gold_spawn", "Resources/Resources/G_Spawn.png", SpriteInfo::animated(rect(0, 0, 896, 128), size(128, 128))),
    //("meat_spawn", "Resources/Resources/M_Spawn.png", SpriteInfo::animated(rect(0, 0, 896, 128), size(128, 128))),
    //("wood_spawn", "Resources/Resources/W_Spawn.png", SpriteInfo::animated(rect(0, 0, 896, 128), size(128, 128))),
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

struct PackSprite {
    index: u32,
    size: Size,
    rect: Rect,
}

struct PackingState<'a> {
    sprites: &'a mut [PackSprite],
    processed: &'a mut [bool],
    area: Rect,
}

impl<'a> PackingState<'a> {
    fn store_next_sprite(&mut self) -> Option<Size> {
        let index = self.processed.iter_mut().enumerate()
            .position(|(index, processed)| {
                if *processed == false {
                    let size = self.sprites[index].size;
                    self.area.fits(size.width, size.height)
                } else {
                    false
                }
            })?;

        let pack = &mut self.sprites[index];
        let size = pack.size;
        pack.rect = Rect { left: self.area.left, top: self.area.top, right: self.area.left + size.width, bottom: self.area.top + size.height };

        self.processed[index] = true;

        Some(size)
    }

    fn has_remaining_items(&self) -> bool {
        self.processed.iter().any(|processed| *processed )
    }
}

/// Maps the state data into a format that can be processed by `pack_sprites`
fn generate_pack_sprites(state: &mut AssetsState) -> Vec<PackSprite> {
    fn sort_sprites(sprite1: &PackSprite, sprite2: &PackSprite) -> Ordering {
        if sprite2.size.height < sprite1.size.height {
            return Ordering::Less;
        } else if sprite2.size.height > sprite1.size.height {
            return Ordering::Greater;
        } else {
            return sprite2.size.width.cmp(&sprite1.size.width);
        }
    }

    let mut sprites: Vec<PackSprite> = state.sprites_data.iter().enumerate()
        .map(|(index, sprite)| PackSprite { 
            index: index as u32,
            size: sprite.area.size(),
            rect: Default::default()
        })
        .collect();

    sprites.sort_unstable_by(sort_sprites);

    sprites
}

/// Fill a row with sprites
fn pack_row(mut state: PackingState) -> bool {
    loop {
        let size = match state.store_next_sprite() {
            Some(value) => value,
            None => { return state.has_remaining_items(); }
        };

        if size.height <= state.area.height() {
            let state = PackingState {
                sprites: state.sprites,
                processed: state.processed,
                area: Rect {
                    left: state.area.left,
                    top: state.area.top + size.height,
                    right: state.area.left + size.width,
                    bottom: state.area.bottom,
                }
            };
            if !pack_row(state) {
                return false;
            }
        }

        state.area.left += size.width;
    }
}

/// Compute the position of the sprites in the final atlas
fn pack_sprites(state: &mut AssetsState, mut sprites: Vec<PackSprite>) {
    let mut processed = vec![false; sprites.len()];
    let mut top = 0;

    loop {
        let size = match processed.iter().enumerate().find(|(_, processed)| **processed == false ) {
            Some((index, _)) => sprites[index].size,
            None => { break; }
        };

        let pack_state = PackingState {
            sprites: &mut sprites,
            processed: &mut processed,
            area: rect(0, top, DST_WIDTH as u32, top+size.height)
        };
        if !pack_row(pack_state) {
            break;
        }

        top += size.height;
    }

    // Copy the generated packed rect into the asset state
    for sprite in sprites {
        state.sprites_dst[sprite.index as usize] = sprite.rect;
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

    for (i, sprite) in state.sprites_data.iter().enumerate() {
        let dst_rect = state.sprites_dst[i];
        let dst_x = dst_rect.left as usize;
        let dst_y = dst_rect.top as usize;
        let height = sprite.area.height() as usize;
        let src_stride = sprite.line_size();
        copy_sprite(
            &mut dst_bytes, dst_x, dst_y, dst_stride,
            &sprite.pixels, src_stride, height,
        );
    }

    state.output_image_bytes = dst_bytes;
}

fn generate_tilemap(state: &mut AssetsState) {
    let sprites = generate_pack_sprites(state);
    pack_sprites(state, sprites);
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

    for (i, name) in state.sprite_names.iter().enumerate() {
        let [left, top, right, bottom] = state.sprites_dst[i].splat();
        csv_out.push_str(&format!("{};{};{};{};{};\n", name, left, top, right, bottom));
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
