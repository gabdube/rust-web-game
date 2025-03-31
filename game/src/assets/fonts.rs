use crate::error::Error;
use crate::shared::{AABB, Size, size};


#[repr(C)]
#[derive(Default, Copy, Clone)]
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

#[derive(Copy, Clone, Default, Debug)]
pub struct ComputedGlyph {
    pub position: AABB,
    pub texcoord: AABB,
}

#[derive(Default)]
pub struct TextMetrics {
    pub size: Size<f32>,
    pub glyphs: Vec<ComputedGlyph>,
}

#[derive(Default)]
pub struct FontAsset {
    pub info: AtlasInfo,
    pub glyphs: Vec<AtlasGlyph>,
    pub texture_id: u32,
}

impl FontAsset {

    pub fn from_bytes(texture_id: u32, bytes: &[u8]) ->  Result<Self, Error> {
        let (x, _, y) = unsafe { bytes.align_to::<u32>() };
        if x.len() != 0 || y.len() != 0 {
            return Err(assets_err!("Failed to parse font atlas data. Data must be aligned to 4 bytes"));
        }

        let info = unsafe { *(bytes.as_ptr() as *const AtlasInfo) };

        let glyph_ptr = unsafe { bytes.as_ptr().add(size_of::<AtlasInfo>()) as *const AtlasGlyph };
        let mut glyphs = vec![Default::default(); info.glyph_max as usize];
        for i in 0..(info.glyph_count as usize) {
            let glyph: AtlasGlyph = unsafe { glyph_ptr.add(i).read() };
            glyphs[glyph.unicode as usize] = glyph;
        }

        let data = FontAsset {
            info,
            glyphs,
            texture_id,
        };

        Ok(data)
    }

    /// Compute the bounds of character `c` at scale `scale` into `glyph`. Return the advance of the glyph
    pub fn compute_glyph(&self, c: &str, scale: f32, glyph: &mut ComputedGlyph) -> f32 {
        // Multi characters glyph not supported
        let chr = match c.len() == 1 {
            true => c.chars().next().unwrap_or('?'),
            false => '?'
        };

        let atlas_height = self.info.height;
        let atlas_glyph = self.glyphs.get(chr as usize).copied().unwrap_or_default();

        glyph.position.left = scale * atlas_glyph.plane_bound[0];
        glyph.position.top = scale * atlas_glyph.plane_bound[3];
        glyph.position.right = scale * atlas_glyph.plane_bound[2];
        glyph.position.bottom = scale * atlas_glyph.plane_bound[1];

        glyph.texcoord.left = atlas_glyph.atlas_bound[0];
        glyph.texcoord.top = atlas_height - atlas_glyph.atlas_bound[3];
        glyph.texcoord.right = atlas_glyph.atlas_bound[2];
        glyph.texcoord.bottom = atlas_height - atlas_glyph.atlas_bound[1];

        atlas_glyph.advance * scale
    }

    pub fn compute_text_metrics(&self, text: &str, scale: f32) -> TextMetrics {
        use unicode_segmentation::UnicodeSegmentation;
        
        let mut glyphs = Vec::with_capacity(text.len());
        let mut advance = 0.0;
        let mut max_height = 0.0;
        let mut glyph = ComputedGlyph::default();
        for g in text.graphemes(true) {
            let a = self.compute_glyph(g, scale, &mut glyph);
            glyph.position.left += advance;
            glyph.position.right += advance;
    
            advance += a;
            max_height = f32::max(max_height, glyph.position.bottom);

            glyphs.push(glyph);
        }

        // Second pass to align the glyph on the bottom
        // This also flips the y axis
        for glyph in glyphs.iter_mut() {
            glyph.position.top = max_height - glyph.position.top;
            glyph.position.bottom = max_height - glyph.position.bottom;
        }

        let size = match text.len() {
            0 => size(0.0, 0.0),
            _ => size(glyph.position.right, max_height)
        };

        TextMetrics { 
            size,
            glyphs
        }
    }

}


impl crate::store::SaveAndLoad for FontAsset {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write(&self.info);
        writer.write_slice(&self.glyphs);
        writer.write_u32(self.texture_id);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        FontAsset { 
            info: reader.read(),
            glyphs: reader.read_vec(),
            texture_id: reader.read_u32(),
        }
    }

}

impl crate::store::SaveAndLoad for TextMetrics {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write(&self.size);
        writer.write_slice(&self.glyphs);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let size: Size<f32> = reader.read();
        let glyphs: Vec<ComputedGlyph> = reader.read_vec();
        TextMetrics {
            size,
            glyphs
        }
    }
}

