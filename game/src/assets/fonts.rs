use crate::error::Error;
use crate::shared::AABB;


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

#[derive(Copy, Clone, Default)]
pub struct ComputedGlyph {
    pub position: AABB,
    pub texcoord: AABB,
}

#[derive(Default)]
pub struct FontAtlasData {
    pub info: AtlasInfo,
    pub glyphs: Vec<AtlasGlyph>,
    pub texture_id: u32,
}

impl FontAtlasData {

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

        let data = FontAtlasData {
            info,
            glyphs,
            texture_id,
        };

        Ok(data)
    }

}


impl crate::store::SaveAndLoad for FontAtlasData {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write(&self.info);
        writer.write_slice(&self.glyphs);
        writer.write_u32(self.texture_id);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        FontAtlasData { 
            info: reader.read(),
            glyphs: reader.read_slice().to_vec(),
            texture_id: reader.read_u32(),
        }
    }

}
