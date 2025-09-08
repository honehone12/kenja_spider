use std::{collections::{HashMap, VecDeque}, io::Cursor};
use image::{GenericImageView, ImageReader};
use tokio::fs;
use mongodb::Client as MongoClient;
use reqwest::Client as HttpClient;
use fantoccini::{elements::Element, wd::Locator, Client as WebDriverClient, ClientBuilder};
use serde_json::{Map, json};
use bytes::Bytes;
use tracing::warn;
use url::Url;
use http::StatusCode;
use anyhow::{Result, bail};

const UA: &str = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:141.0) Gecko/20100101 Firefox/141.0";
const MAX_W: u32 = 256;
const MAX_H: u32 = 512;

pub struct Spider<'a> {
    mongo_client: MongoClient,
    http_client: HttpClient,
    web_driver_client: WebDriverClient,
    img_root: &'a str
}

pub struct InitParams<'a> {
    pub mongo_uri: &'a str, 
    pub web_driver_uri: &'a str,
    pub img_root: &'a str
}

pub struct CrawlParams<'a> {
    pub mongo_db: &'a str,
    pub mongo_cl: &'a str,
    pub target_id: i64,
    pub target_url: &'a str
}

struct CrawlOneOutput<'a> {
    found_urls: Vec<&'a str>
}

struct ImgReqOutput {
    img_name: String,
    img_path: String,
    body: Bytes
}

impl<'a> Spider<'a> {
    pub async fn new(params: InitParams<'a>) -> Result<Self> {
        let mongo_client = MongoClient::with_uri_str(params.mongo_uri).await?;
        let http_client = HttpClient::builder().user_agent(UA).build()?;
        
        let mut cap = Map::new();
        cap.insert("moz:firefoxOptions".to_string(), json!({
            "args": ["-headless"],
            "log": json!({"level": "error"})
        }));

        let web_driver_client = ClientBuilder::native()
            .capabilities(cap)
            .connect(params.web_driver_uri).await?;
        web_driver_client.set_ua(UA).await?;

        if !fs::try_exists(params.img_root).await? {
            fs::create_dir_all(params.img_root).await?;
        }

        Ok(Self {
            mongo_client,
            http_client,
            web_driver_client,
            img_root: params.img_root
        })
    }

    fn is_supported_file(name: &str) -> bool {
        if name.starts_with("data:") {
            return false;
        }
        if !name.ends_with(".jpg") && !name.ends_with(".png") && !name.ends_with(".webp") {
            return false;    
        }

        true
    }

    fn rename_img(name: &str) -> Result<String> {
        let Some(ex) = name.split('.').last() else {
            bail!("could not find file extension");
        };
        let hash = blake3::hash(name.as_bytes());
        let hashed_name = format!("{}.{ex}", hex::encode(hash.as_bytes()));
        Ok(hashed_name)
    }

    fn write_resized_img(img_raw: Bytes, path: &str) -> Result<()> {
        let cursor = Cursor::new(img_raw.to_vec());
        let mut img = ImageReader::new(cursor).decode()?;
        let (w, h) = img.dimensions();
        if w > MAX_W || h > MAX_H {
            img = img.thumbnail(MAX_W, MAX_H);
        }

        img.save(path)?;

        Ok(())
    }

    pub async fn crawl(&self, params: CrawlParams<'a>) -> Result<()> {
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

            let output = self.crawl_one(next, self.img_root).await?;
            
            q.extend(output.found_urls);
            crawled_map.insert(params.target_url, true);
        }

        Ok(())
    }

    async fn crawl_one(&self, target_url: &str, img_root: &str) -> Result<CrawlOneOutput<'a>> 
    {
        let url = Url::parse(target_url)?;

        self.web_driver_client.goto(target_url).await?;
        self.web_driver_client.wait().for_url(&url).await?;

        let scraped_imgs = self.scrape_imgs(&url).await?;

        Ok(CrawlOneOutput {
            found_urls: vec![]
        })
    }

    async fn scrape_imgs(&self, current_url: &Url) -> Result<Vec<String>> {
        let img_tags = self.web_driver_client.find_all(Locator::Css("img")).await?;
        let mut output = vec![];

        for img in img_tags {
            let img_out = match self.req_img(img, current_url).await {
                Err(e) => {
                    warn!("{e}");
                    continue;
                },
                Ok(None) => continue,
                Ok(Some(o)) => o
            };

            Self::write_resized_img(img_out.body, &img_out.img_path)?;
            output.push(img_out.img_name);
        }

        Ok(output)
    }

    async fn req_img(&self, img: Element, current_url: &Url) -> Result<Option<ImgReqOutput>> {
        let Some(src) = img.attr("src").await? else {
            bail!("could not find img src");
        };

        if !Self::is_supported_file(&src) {
            bail!("unsupported file");
        }

        let img_name = Self::rename_img(&src)?;
        let img_path = format!("{}/{img_name}", self.img_root);
        if fs::try_exists(&img_path).await? {
            return Ok(None);
        } 

        let url = current_url.join(&src)?;
        let res = self.http_client.get(url).send().await?;
        if res.status() != StatusCode::OK {
            bail!("failed to fetch image [{}] {src}", res.status());
        }
        let body = res.bytes().await?;

        Ok(Some(ImgReqOutput { 
            img_name, 
            img_path, 
            body 
        }))
    }
}
