use fnv::FnvHashMap;
use crate::error::Error;

#[derive(Copy, Clone)]
pub struct Texture {
    // The unique ID of the texture. It is also the index of the texture array in the engine data
    pub id: u32,
}

pub struct Assets {
    pub textures: FnvHashMap<String, Texture>,
}

impl Assets {

    pub fn load_bundle(&mut self, bundle_raw: &str) -> Result<(), Error> {
        let mut error = None;

        Self::split_csv(&bundle_raw, |args| {
            let result = match args[0] {
                "TEXTURE" => {
                    self.load_texture(args)
                },
                "JSON" => {
                    Ok(())
                },
                "SHADER" => Ok(()),
                _ => { Err(assets_err!("Unknown asset type {:?}", args[0])) }
            };

            if let Err(e1) = result {
                if error.is_none() {
                    error = Some(e1);
                } else {
                    let e2 = error.as_mut().unwrap();
                    e2.merge(e1);
                }
            }
        });

        if let Some(err) = error {
            return Err(err);
        }

        Ok(())
    }

    fn split_csv<CB: FnMut(&[&str])>(csv: &str, mut callback: CB) {
        let mut start = 0;
        let mut end = 0;
        let mut chars_iter = csv.chars();
        let mut args: [&str; 8] = [""; 8];
        while let Some(c) = chars_iter.next() {
            end += 1;
            if c == '\n' {
                let line = &csv[start..end];
                let mut args_count = 0;
                for substr in line.split(';') {
                    args[args_count] = substr;
                    args_count += 1;
                }

                if args_count > 1 {
                    callback(&args[0..(args_count-1)]);
                }

                start = end;
            }
        }
    }

    fn load_texture(&mut self, args: &[&str]) -> Result<(), Error> {
        let name = match args.get(1) {
            Some(name) => name.to_string(),
            None => { return Err(assets_err!("Missing texture name")); }
        };

        let id = self.textures.len() as u32;
        self.textures.insert(name, Texture { id });

        Ok(())
    }
}

impl crate::store::SaveAndLoad for Assets {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_string_hashmap(&self.textures);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let mut assets = Assets::default();
        assets.textures = reader.read_string_hashmap();
        assets
    }
}

impl Default for Assets {

    fn default() -> Self {
        Assets {
            textures: FnvHashMap::default(),
        }
    }

}
