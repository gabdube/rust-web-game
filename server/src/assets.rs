use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

enum AssetData {
    Bin(Box<[u8]>),
    Text(String),
}

pub(crate) struct Asset {
    mime: String,
    data: AssetData
}

impl Asset {

    pub fn mime(&self) -> &str {
        &self.mime
    }

    pub fn bytes(&self) -> Box<[u8]> {
        match &self.data {
            AssetData::Bin(bin) => bin.clone(),
            AssetData::Text(text) => text.as_bytes().to_vec().into_boxed_slice(),
        }
    }

}

struct InnerAssetCache {
    files: HashMap<PathBuf, Arc<Asset>>,
    reload: HashSet<PathBuf>,
}

impl InnerAssetCache {

    fn cache_fetch(&self, path: &Path) -> Option<Arc<Asset>> {
        self.files.get(path).cloned()
    }

    fn cache_load(&mut self, path: PathBuf) -> Option<Arc<Asset>> {
        let mime = crate::utils::mime_from_path(&path);
        let data = match crate::utils::is_text(&mime) {
            true => AssetData::Text(::std::fs::read_to_string(&path).ok()?),
            false => AssetData::Bin(::std::fs::read(&path).ok()?.into_boxed_slice())
        };

        let asset = Arc::new(Asset {
            mime,
            data
        });

        println!("Loaded {:?}", path);
        self.files.insert(path, Arc::clone(&asset));

        Some(asset)
    }
}


pub(crate) struct AssetCache {
    inner: Arc<Mutex<InnerAssetCache>>
}

impl AssetCache {

    pub fn new() -> Self {
        let inner = InnerAssetCache {
            files: HashMap::with_capacity(16),
            reload: HashSet::with_capacity(4)
        };

        AssetCache {
            inner: Arc::new(Mutex::new(inner))
        }
    }

    pub fn fetch_asset(&self, path: PathBuf) -> Option<Arc<Asset>> {
        let mut inner = self.inner.lock().unwrap();
        inner.cache_fetch(&path)
            .or_else(|| inner.cache_load(path) )
    }

    pub fn reload_asset(&self, path: PathBuf) {
        let mut inner = self.inner.lock().unwrap();
        inner.reload.insert(crate::utils::system_path_to_web_path(&path));
        inner.cache_load(path);
    }

    pub fn asset_loaded(&self, path: &Path) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.files.contains_key(path)
    }

    pub fn updated_files(&self) -> Vec<PathBuf> {
        let mut inner = self.inner.lock().unwrap();
        inner.reload.drain().collect()
    }
    
}

impl Clone for AssetCache {
    fn clone(&self) -> Self {
        AssetCache {
            inner: Arc::clone(&self.inner),
        }
    }
}
