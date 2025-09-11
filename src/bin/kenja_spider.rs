use std::{env, time::Duration};
use tokio::fs;
use clap::Parser;
use kenja_spider::{
    spider::{CrawlParams, InitParams, Spider}, 
    documents::{Size, UrlSrc}
};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    list: String,
    #[arg(long, default_value_t = 256)]
    width: u32,
    #[arg(long, default_value_t = 512)]
    height: u32,
    #[arg(long, default_value_t = 10)]
    timeout_sec: u64,
    #[arg(long, default_value_t = 1)]
    interval_sec: u64
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv()?;
    let args = Args::parse();
    let json = fs::read_to_string(args.list).await?;
    let target_list = serde_json::from_str::<Vec<UrlSrc>>(&json)?;

    let params = InitParams{ 
        mongo_uri: &env::var("MONGO_URI")?, 
        web_driver_uri: "http://localhost:4444",
        user_agent: &env::var("SPIDER_UA")?, 
        image_root: &env::var("IMG_ROOT")?,
        size: Size{
            w: args.width,
            h: args.height
        },
        timeout: Duration::from_secs(args.timeout_sec),
        interval: Duration::from_secs(args.interval_sec) 
    };
    let spider = Spider::new(params).await?;

    let params = CrawlParams { 
        mongo_db: &env::var("SPIDER_DB")?, 
        mongo_cl: &env::var("SPIDER_CL")?, 
        target_list 
    };
    spider.crawl(params).await?;

    Ok(())
}
