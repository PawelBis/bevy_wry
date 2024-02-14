use bevy::prelude::UVec2;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Deserialize)]
pub enum EditorCommand {
    ResizeViewport {
        #[serde(deserialize_with = "deserialie_new_size")]
        new_position: Option<UVec2>,
        #[serde(deserialize_with = "deserialie_new_size")]
        new_size: Option<UVec2>,
    },
}

/// Deserialize for Option<UVec2> - EXPENSIVE
/// This is for Json only, in the future we will use Protobuf and this anyway.
fn deserialie_new_size<'de, D>(d: D) -> Result<Option<UVec2>, D::Error>
where D: Deserializer<'de> {
    let map = match serde_json::Map::deserialize(d){
        Ok(m) => m,
        Err(_) => return Ok(None),
    };

    let x: u32 = serde_json::from_value(map.get("x").unwrap().clone()).unwrap();
    let y: u32 = serde_json::from_value(map.get("y").unwrap().clone()).unwrap();

    Ok(Some(UVec2 {x, y}))
}

