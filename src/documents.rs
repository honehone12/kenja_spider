use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Parent {
    pub name: String,
    pub name_japanese: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct UrlSrc {
    pub mal_id: i64,
    pub url: String,
    pub parent: Parent
}

#[derive(Serialize, Deserialize)]
pub struct UrlCheckOutput {
    pub mal_id: i64,
    pub url: String,
    pub https: Option<String>,
    pub http: Option<String>,
    pub parent: Parent
}

#[derive(Serialize, Deserialize)]
pub struct SpiderOutput {
    pub mal_id: i64,
    pub url: String,
    pub images: Vec<String>,
    pub videos: Vec<String>,
    pub parent: Parent
}

#[derive(Clone, Copy)]
pub struct Size {
    pub w: u32,
    pub h: u32
}
