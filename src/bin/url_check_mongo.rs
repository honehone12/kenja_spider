use std::env;
use mongodb::Client as MongoClient;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct UrlSrc {

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

    Ok(())
}
