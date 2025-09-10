use std::{env, time::Duration};
use clap::Parser;
use kenja_spider::spider::{CrawlParams, InitParams, Size, Spider};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    id: i64,
    #[arg(long)]
    url: String,
    #[arg(long, default_value_t = 256)]
    width: u32,
    #[arg(long, default_value_t = 512)]
    height: u32,
    #[arg(long, default_value_t = 1)]
    interval_sec: u64
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv()?;

    let args = Args::parse();
    let mongo_uri = env::var("MONGO_URI")?;
    let params = InitParams{ 
        mongo_uri: &mongo_uri, 
        web_driver_uri: "http://localhost:4444",
        user_agent: &env::var("SPIDER_UA")?, 
        image_root: &env::var("IMG_ROOT")? 
    };
    let spider = Spider::new(params).await?;

    let params = CrawlParams { 
        mongo_db: &env::var("SPIDER_DB")?, 
        mongo_cl: &env::var("SPIDER_CL")?, 
        target_id: args.id, 
        target_url: &args.url,
        size: Size{
            w: args.width,
            h: args.height
        },
        interval: Duration::from_secs(args.interval_sec) 
    };
    spider.crawl(params).await?;

    Ok(())
}
