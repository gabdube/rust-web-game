use std::path::PathBuf;
use hyper_tungstenite::tungstenite::Message;
use serde_json::{Map, Value};


/// Message sent by the engine to the client
#[derive(Copy, Clone)]
pub enum EngineMessageType {
    FileChanged = 1,
}

impl EngineMessageType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            EngineMessageType::FileChanged => "FILE_CHANGED",
        }
    }

    
}

impl Into<serde_json::Value> for EngineMessageType {
    fn into(self) -> serde_json::Value {
        serde_json::Value::String(self.as_str().to_string())
    }
}

//
// Websocket messages
//

pub fn file_changed(changed: PathBuf) -> Message {
    let mut json = Value::Object(Map::with_capacity(2));
    json["name"] = EngineMessageType::FileChanged.into();
    json["data"] = changed.to_string_lossy().into();
    Message::Text(serde_json::to_string(&json).unwrap())
}
