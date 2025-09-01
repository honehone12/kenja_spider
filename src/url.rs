use std::time::Duration;
use tokio::time;
use reqwest::{Client as HttpClient, StatusCode};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct UrlSrc {
    pub src: String,
    pub url: String
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
    if url_src.url.starts_with("https:") {
        let res = http_client.get(&url_src.url).send().await?;
        if res.status() == StatusCode::OK {
            return Ok(UrlOut{
                src: url_src.src,
                original: url_src.url.clone(),
                https: Some(url_src.url),
                http: None
            })
        }
    } else if url_src.url.starts_with("http:") {
        let tls = url_src.url.replace("http:", "https:");
        let res = http_client.get(&tls).send().await?;
        if res.status() == StatusCode::OK {
            return Ok(UrlOut{
                src: url_src.src,
                original: url_src.url.clone(),
                https: Some(url_src.url),
                http: None
            })
        }

        time::sleep(SLEEP).await;

        let res = http_client.get(&url_src.url).send().await?;
        if res.status() == StatusCode::OK {
            return Ok(UrlOut{
                src: url_src.src,
                original: url_src.url.clone(),
                https: None,
                http: Some(url_src.url)
            })
        }
    }

    time::sleep(SLEEP).await;

    Ok(UrlOut{
        src: url_src.src,
        original: url_src.url,
        https: None,
        http: None
    })
}
