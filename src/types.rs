use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum EditorCommand {
    ResizeViewport {
        new_width: Option<u32>,
        new_height: Option<u32>,
    },
}
