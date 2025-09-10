use std::time::Duration;
use reqwest::{Client as HttpClient, StatusCode};
use tracing::info;
use crate::documents::{UrlSrc, UrlCheckOutput};

const TIMEOUT: Duration = Duration::from_secs(3);

pub async fn check_url(url_src: UrlSrc, http_client: &HttpClient) -> anyhow::Result<UrlCheckOutput> {
    if url_src.url.starts_with("https:") {
        info!("checking {}", url_src.url);
        if let Ok(res) = http_client.get(&url_src.url).timeout(TIMEOUT).send().await {
            if res.status() == StatusCode::OK {
                return Ok(UrlCheckOutput{
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
                return Ok(UrlCheckOutput{
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
                return Ok(UrlCheckOutput{
                    mal_id: url_src.mal_id,
                    url: url_src.url.clone(),
                    https: None,
                    http: Some(url_src.url),
                    parent: url_src.parent
                })
            }
        }
    }

    Ok(UrlCheckOutput{
        mal_id: url_src.mal_id,
        url: url_src.url,
        https: None,
        http: None,
        parent: url_src.parent
    })
}
