use std::env;
use futures::TryStreamExt;
use mongodb::{bson::doc, Client as MongoClient};
use reqwest::Client as HttpClient;
use tokio::fs;
use tracing::info;
use kenja_spider::url::*;

async fn url_check_mongo(
    http_client: HttpClient,
    src_cl: mongodb::Collection<UrlSrc>
) -> anyhow::Result<()> {
    info!("Obtaining documents...");
    let src_list = src_cl.find(doc! {}).await?.try_collect::<Vec<UrlSrc>>().await?;
    let mut https_out = vec![];
    let mut http_out = vec![];
    let mut unknown_out = vec![];

    for url_src in src_list {
        let out = check_url(url_src, &http_client).await?;
        
        if out.https.is_some() {
            https_out.push(out);
        } else if out.http.is_some() {
            http_out.push(out);
        } else {
            unknown_out.push(out);
        }
    }

    info!(
        "done. https: {}, http: {}, unknown: {}",
        https_out.len(),
        http_out.len(),
        unknown_out.len()
    );

    let json = serde_json::to_string_pretty(&https_out)?;
    fs::write("https.json", json).await?;
    let json = serde_json::to_string_pretty(&http_out)?;
    fs::write("http.json", json).await?;
    let json = serde_json::to_string_pretty(&unknown_out)?;
    fs::write("unknown.json", json).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv()?;

    let mongo_uri = env::var("MONGO_URI")?;
    let mongo_client = MongoClient::with_uri_str(mongo_uri).await?;
    let src_db = mongo_client.database(&env::var("URL_SRC_DB")?);
    let src_cl = src_db.collection::<UrlSrc>(&env::var("URL_SRC_CL")?);
    let http_client = HttpClient::new();

    url_check_mongo(http_client, src_cl).await
}
