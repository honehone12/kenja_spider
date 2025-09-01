use tokio::fs;
use kenja_spider::url::UrlOut;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let json = fs::read_to_string("unknown.json").await?;
    let list = serde_json::from_str::<Vec<UrlOut>>(&json)?;

    let clean = list.into_iter().filter(|o| !o.original.is_empty())
        .collect::<Vec<UrlOut>>();
    println!("{} items", clean.len());

    let json = serde_json::to_string_pretty(&clean)?;
    fs::write("unknown.json", json).await?;


    Ok(())
} 