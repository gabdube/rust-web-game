use std::path::{Path, PathBuf, Component};

pub(crate) fn web_path_to_local_path(path: &str) -> PathBuf {
    if path == "/" {
        return PathBuf::from(format!("{}/index.html", super::BUILD_PATH));
    }

    let mut base = PathBuf::from("");

    for subpath in Path::new(path).components() {
        match subpath {
            Component::RootDir => { base.push(super::BUILD_PATH); },
            Component::Normal(p) => { base.push(p); }
            _ => {},
        }
    }

    base
}

pub(crate) fn system_path_to_local_path(path: &Path) -> Option<PathBuf> {
    let mut base = PathBuf::from(super::BUILD_PATH);

    let mut base_found = false;
    for subpath in path.components() {
        if base_found {
            base.push(subpath);
        } else {
            base_found = subpath.as_os_str() == "build";
        }
    }

    match base_found {
        true => Some(base),
        false => None
    }
}

pub(crate) fn system_path_to_web_path(path: &Path) -> PathBuf {
    let mut path_string = path.to_string_lossy().to_string();
    path_string = path_string.replace(super::BUILD_PATH, "");
    path_string = path_string.replace("\\", "/");
    PathBuf::from(path_string)
}

pub(crate) fn mime_from_path(path: &Path) -> String {
    let ext = path.extension().map(|ext| ext.to_str().unwrap() ).unwrap_or("");
    let mime = match ext {
        "html" => "text/html",
        "js" => "application/x-javascript",
        "wasm" => "application/wasm",
        "json" => "application/json",
        _ => "application/octet-stream"
    };

    mime.to_string()
}    

pub(crate) fn is_text(mime: &str) -> bool {
    match mime {
        "text/html" | "application/x-javascript" | "application/json" => true,
        _ => false
    }
}
