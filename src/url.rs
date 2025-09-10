use std::time::Duration;
use reqwest::{Client as HttpClient, StatusCode};
use serde::{Serialize, Deserialize};
use tracing::info;

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
pub struct UrlOut {
    pub mal_id: i64,
    pub url: String,
    pub https: Option<String>,
    pub http: Option<String>,
    pub parent: Parent
}

const TIMEOUT: Duration = Duration::from_secs(3);

pub async fn check_url(url_src: UrlSrc, http_client: &HttpClient) -> anyhow::Result<UrlOut> {
    if url_src.url.starts_with("https:") {
        info!("checking {}", url_src.url);
        if let Ok(res) = http_client.get(&url_src.url).timeout(TIMEOUT).send().await {
            if res.status() == StatusCode::OK {
                return Ok(UrlOut{
                    mal_id: url_src.mal_id,
                    url: url_src.url.clone(),
                    https: Some(url_src.url),
                    http: None,
                    parent: url_src.parent
                })
            }
        }
    } else if url_src.url.starts_with("http:") {
        let tls = url_src.url.replace("http:", "https:");
        info!("checking {tls}");
        if let Ok(res) = http_client.get(&tls).timeout(TIMEOUT).send().await {
            if res.status() == StatusCode::OK {
                return Ok(UrlOut{
                    mal_id: url_src.mal_id,
                    url: url_src.url,
                    https: Some(tls),
                    http: None,
                    parent: url_src.parent
                })
            }
        }

        info!("checking {}", url_src.url);
        if let Ok(res) = http_client.get(&url_src.url).timeout(TIMEOUT).send().await {
            if res.status() == StatusCode::OK {
                return Ok(UrlOut{
                    mal_id: url_src.mal_id,
                    url: url_src.url.clone(),
                    https: None,
                    http: Some(url_src.url),
                    parent: url_src.parent
                })
            }
        }
    }

    Ok(UrlOut{
        mal_id: url_src.mal_id,
        url: url_src.url,
        https: None,
        http: None,
        parent: url_src.parent
    })
}
