use std::time::Duration;
use tokio::time;
use reqwest::{Client as HttpClient, StatusCode};
use serde::{Serialize, Deserialize};
use tracing::info;

#[derive(Serialize, Deserialize)]
pub struct UrlSrc {
    pub src: String,
    pub url: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct UrlOut {
    pub src: String,
    pub original: String,
    pub https: Option<String>,
    pub http: Option<String>
}

const SLEEP: Duration = Duration::from_millis(100);

pub async fn check_url(url_src: UrlSrc, http_client: &HttpClient) -> anyhow::Result<UrlOut> {
    let url = match url_src.url {
        Some(s) => s,
            None => return Ok(UrlOut{
            src: url_src.src,
            original: "".to_string(),
            https: None,
            http: None
        })
    };

    if url.starts_with("https:") {
        info!("checking {}", url);
        let res = http_client.get(&url).send().await?;
        if res.status() == StatusCode::OK {
            return Ok(UrlOut{
                src: url_src.src,
                original: url.clone(),
                https: Some(url),
                http: None
            })
        }
    } else if url.starts_with("http:") {
        let tls = url.replace("http:", "https:");
        info!("checking {tls}");
        let res = http_client.get(&tls).send().await?;
        if res.status() == StatusCode::OK {
            return Ok(UrlOut{
                src: url_src.src,
                original: url.clone(),
                https: Some(url),
                http: None
            })
        }

        time::sleep(SLEEP).await;

        info!("checking {}", url);
        let res = http_client.get(&url).send().await?;
        if res.status() == StatusCode::OK {
            return Ok(UrlOut{
                src: url_src.src,
                original: url.clone(),
                https: None,
                http: Some(url)
            })
        }
    }

    time::sleep(SLEEP).await;

    Ok(UrlOut{
        src: url_src.src,
        original: url,
        https: None,
        http: None
    })
}
