use std::collections::{HashMap, VecDeque};
use mongodb::Client as MongoClient;
use reqwest::Client as HttpClient;
use fantoccini::{Client as WebDriverClient, ClientBuilder};
use serde_json::{Map, json};

const UA: &str = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:141.0) Gecko/20100101 Firefox/141.0";

pub struct Spider {
    mongo_client: MongoClient,
    http_client: HttpClient,
    web_driver_client: WebDriverClient
}

pub struct SpiderParams<'a> {
    pub mongo_db: &'a str,
    pub mongo_cl: &'a str,
    pub target_id: i64,
    pub target_url: &'a str
}

struct CrawlOneOutput<'a> {
    pub found_urls: Vec<&'a str>
}

impl<'a> Spider {
    pub async fn new(mongo_uri: &str, web_driver_uri: &str) -> anyhow::Result<Self> {
        let mongo_client = MongoClient::with_uri_str(mongo_uri).await?;
        let http_client = HttpClient::builder().user_agent(UA).build()?;
        
        let mut cap = Map::new();
        cap.insert("moz:firefoxOptions".to_string(), json!({
            "args": ["-headless"],
            "log": json!({"level": "error"})
        }));

        let web_driver_client = ClientBuilder::native()
            .capabilities(cap)
            .connect(web_driver_uri).await?;
        web_driver_client.set_ua(UA).await?;

        Ok(Self {
            mongo_client,
            http_client,
            web_driver_client
        })
    }

    pub async fn crawl(&self, params: SpiderParams<'a>) -> anyhow::Result<()> {
        let mut crawled_map = HashMap::new();

        let mut q = VecDeque::new();
        q.push_back(params.target_url);

        loop {
            let Some(next) = q.pop_front() else {
                break;
            };

            if let Some(true) = crawled_map.get(next) {
                continue;
            }


            let output = self.crawl_one(next).await?;
            
            q.extend(output.found_urls);
            crawled_map.insert(params.target_url, true);
        }

        Ok(())
    }

    async fn crawl_one(&self, target_url: &str) -> anyhow::Result<CrawlOneOutput> {

        Ok(CrawlOneOutput {
            found_urls: vec![]
        })
    }
}
