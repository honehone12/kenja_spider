use std::env;
use clap::Parser;
use kenja_spider::spider::{Spider, SpiderParams};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    id: i64,
    #[arg(long)]
    url: String
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv()?;

    let args = Args::parse();
    let mongo_uri = env::var("MONGO_URI")?;

    let spider = Spider::new(&mongo_uri, "http://localhost:4444").await?;

    let params = SpiderParams { 
        mongo_db: &env::var("SPIDER_DB")?, 
        mongo_cl: &env::var("SPIDER_CL")?, 
        target_id: args.id, 
        target_url: &args.url 
    };
    spider.crawl(params).await?;

    Ok(())
}
