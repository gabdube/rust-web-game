use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};

use crate::assets::AssetCache;

static EXTENSIONS_TO_RELOAD: &[&str] = &["html", "js", "wasm", "ktx2", "json", "bin", "frag", "vert", "csv"];

fn filter_change_events<'a>(assets: &AssetCache, event: &'a Event, dedup: &Vec<PathBuf>) -> Option<PathBuf> {
    let filter_event_kind = |event: &'a Event| -> Option<&'a Event> {
        match event.kind {
            EventKind::Modify(_) => Some(event),
            _ => None
        }
    };

    let filter_extensions = |event: &'a Event| -> Option<&'a PathBuf> {
        let path = event.paths.first()?;
        let ext = path.extension()?;
        match EXTENSIONS_TO_RELOAD.iter().any(|&ext2| ext == ext2 ) {
            true => Some(path),
            false => None
        }
    };

    let filter_loaded = |path: &PathBuf| -> Option<PathBuf> { 
        let local_path = crate::utils::system_path_to_local_path(&path)?;
        match assets.asset_loaded(&local_path) {
            true => Some(local_path),
            false => None
        }
    };

    let filter_dedup = |path: PathBuf| -> Option<PathBuf> {
        match dedup.iter().any(|p| p == &path) {
            false => Some(path),
            true => None
        }
    };

    Some(event)
        .and_then(filter_event_kind)
        .and_then(filter_extensions)
        .and_then(filter_loaded)
        .and_then(filter_dedup)
}


pub(crate) fn reload_assets(assets: &AssetCache) {
    const WAIT: ::std::time::Duration = ::std::time::Duration::from_millis(200);

    let assets = assets.clone();
    
    thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let path = Path::new(super::BUILD_PATH).canonicalize().unwrap();

        let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
        watcher.watch(path.as_ref(), RecursiveMode::Recursive).unwrap();

        let mut accumulate = Vec::with_capacity(32);
        loop {
            while let Ok(Ok(event)) = rx.recv_timeout(WAIT) {
                if let Some(path) = filter_change_events(&assets, &event, &accumulate) {
                    accumulate.push(path);
                }
            }

            for path in accumulate.drain(..) {
                assets.reload_asset(path);
            }
        }
    });
}
