use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SpiderOutput {
    pub mal_id: i64,
    pub url: String,
    pub images: Vec<String>,
    pub videos: Vec<String>
}
