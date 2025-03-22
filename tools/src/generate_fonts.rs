//! Generate the font atlas
//! Requires msdfgen (https://github.com/Chlumsky/msdf-atlas-gen)
//! Usage: `cargo run -p tools --release -- -c generate_fonts --msdfgen /path/to/msdf-atlas-gen`
use glob::glob;
use std::error::Error;
use std::io::prelude::*;
use std::process::Command;
use std::path::{Path, PathBuf};

fn msdf_gen_path() -> Option<String> {
    let index = ::std::env::args().position(|arg| arg.as_str() == "--msdfgen" )?;
    ::std::env::args().skip(index + 1).next()
}

fn font_output(file_name: &str, ext: &str) -> PathBuf {
    let mut path = PathBuf::new();
    path.push("build/assets/fonts");
    path.push(file_name);
    path.set_extension(ext);
    path
}

fn save_msdf_atlas_rgba(out_path: &Path, image_info: &png::OutputInfo, image_data_rgb: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    use std::io::BufWriter;
    
    if ::std::fs::exists(out_path).unwrap_or(false) {
        ::std::fs::remove_file(out_path)?; // Remove old file
    }

    let file = ::std::fs::File::create_new(out_path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, image_info.width, image_info.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455));
    encoder.set_source_chromaticities(png::SourceChromaticities::new(
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000)
    ));
    let mut writer = encoder.write_header()?;

    let mut image_data_rgba: Vec<[u8; 4]> = vec![[0, 0, 0, 0]; (image_info.width * image_info.height) as usize];
    for (i, chunk) in image_data_rgb.chunks(3).enumerate() {
        image_data_rgba[i] = [chunk[0], chunk[1], chunk[2], 255];
    }

    let (_, bytes, _) = unsafe { image_data_rgba.align_to::<u8>() };
    writer.write_image_data(bytes)?;

    Ok(())
}

fn generate_msdf_atlas(msdf_gen_path: &str, input_font: &Path, output_image: &Path, output_json: &Path) -> Result<(), Box<dyn Error>> {
    let response = Command::new(msdf_gen_path)
        .arg("-font")
        .arg(input_font)
        .arg("-format")
        .arg("png")
        .arg("-json")
        .arg(output_json)
        .arg("-imageout")
        .arg(output_image)
        .arg("-size")
        .arg("35")
        .output()?;

    if response.status.code() == Some(1) {
        return Err(Box::new(StringError(String::from_utf8(response.stderr).unwrap_or(Default::default()))));
    }

    // Need to convert the RBG png into a RGBA to make sure we use an optimized format on all platform
    let src = ::std::fs::File::open(&output_image)
        .map_err(|err| StringError(format!("Failed to open {:?} {:?}", output_image, err)) )?;

    let decoder = png::Decoder::new(src);
    let mut reader = decoder.read_info().unwrap();
    let mut image_data_rgb: Vec<u8> = vec![0; reader.output_buffer_size()];
    let image_info = reader.next_frame(&mut image_data_rgb).unwrap();

    save_msdf_atlas_rgba(&output_image, &image_info, &image_data_rgb)?;

    Ok(())
}

fn compress_atlas_json(json_path: &PathBuf, bin_dst: &PathBuf) -> Result<(), Box<dyn Error>> {
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct AtlasInfo {
        pub size: f32,
        pub width: f32,
        pub height: f32,
        pub line_height: f32,
        pub ascender: f32,
        pub descender: f32,
        pub glyph_count: u32,
        pub glyph_max: u32,
    }

    #[repr(C)]
    #[derive(Copy, Clone, Default)]
    pub struct AtlasGlyph {
        pub unicode: u32,
        pub advance: f32,
        pub atlas_bound: [f32; 4],
        pub plane_bound: [f32; 4],
    }

    fn read_u32(v: &serde_json::Value) -> u32 { v.as_u64().map(|v| v as u32 ).unwrap_or(0) }
    fn read_f32(v: &serde_json::Value) -> f32 { v.as_f64().map(|v| v as f32 ).unwrap_or(0.0f32) }
    fn read_rect(v: &serde_json::Value) -> [f32; 4] {
        match v.as_object() {
            Some(obj) => [
                read_f32(&obj["left"]),
                read_f32(&obj["top"]),
                read_f32(&obj["right"]),
                read_f32(&obj["bottom"])
            ],
            None => [0.0; 4]
        }
    }

    let json_source = ::std::fs::read_to_string(json_path)?;
    let json: serde_json::Value = serde_json::from_str(&json_source).map_err(|err| StringError(format!("Failed to parse json: {:?}", err)) )?;

    let atlas = &json["atlas"];
    let metrics = &json["metrics"];
    let glyphs = &json["glyphs"].as_array().unwrap();
    let mut glyph_max = 0;

    let total_size_u32 = (size_of::<AtlasInfo>() + (size_of::<AtlasGlyph>() * glyphs.len())) / size_of::<u32>();
    let mut output: Vec<u32> = vec![0; total_size_u32];

    // Glyph
    unsafe {
        let glyph_dst_base = output.as_mut_ptr().add(size_of::<AtlasInfo>() / 4) as *mut AtlasGlyph;
        let mut offset: isize = 0;
        for glyph in glyphs.iter() {
            let unicode = read_u32(&glyph["unicode"]);
            glyph_max = u32::max(glyph_max, unicode);

            *glyph_dst_base.offset(offset) = AtlasGlyph {
                unicode,
                advance: read_f32(&glyph["advance"]),
                atlas_bound: read_rect(&glyph["atlasBounds"]),
                plane_bound: read_rect(&glyph["planeBounds"]),
            };
            offset += 1;
        }
    }

    // Info
    unsafe {
        let info_dst = output.as_mut_ptr() as *mut AtlasInfo;
        *info_dst = AtlasInfo {
            size: read_f32(&atlas["size"]),
            width: read_f32(&atlas["width"]),
            height: read_f32(&atlas["height"]),
            line_height: read_f32(&metrics["lineHeight"]),
            ascender: read_f32(&metrics["ascender"]),
            descender: read_f32(&metrics["descender"]),
            glyph_count: glyphs.len() as u32,
            glyph_max: glyph_max + 1,
        };
    }

    let (_, output_bytes, _) = unsafe { output.align_to::<u8>() };
    let mut file = ::std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(bin_dst)
        .unwrap();

    file.write_all(&output_bytes)?;

    // Remove old json file
    ::std::fs::remove_file(json_path)?;

    Ok(())
}

pub fn generate_fonts() {
    let msdf_gen_path = match msdf_gen_path() {
        Some(path) => path,
        None => {
            eprintln!("path to msdf-atlas-gen must be provided using \"--msdfgen /path/to/msdf-atlas-gen\"");
            return;
        }
    };

    for entry in glob("./build/assets/fonts/*.ttf").unwrap().filter_map(Result::ok) {
        println!("Generating msdf atlas for {:?}", entry);

        let file_name = entry.file_name().and_then(|name| name.to_str() ).unwrap_or("");
        let atlas_texture_output = font_output(file_name, "png");
        let atlas_json_output = font_output(file_name, "json");
        if let Err(e) = generate_msdf_atlas(&msdf_gen_path, &entry, &atlas_texture_output, &atlas_json_output) {
            eprintln!("Failed to generate msdf atlas for {:?}: {}", file_name, e);
            continue;
        }

        let atlas_data_output = font_output(&file_name, "bin");
        if let Err(e) = compress_atlas_json(&atlas_json_output, &atlas_data_output) {
            eprintln!("Failed to compress atlas json: {e}");
            continue;
        }
    }
}

#[derive(Debug)]
struct StringError(String);
impl ::std::fmt::Display for StringError { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl Error for StringError {}
