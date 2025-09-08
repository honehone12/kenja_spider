use std::{collections::{HashMap, VecDeque}, io::Cursor};
use image::{GenericImageView, ImageReader};
use tokio::fs;
use mongodb::Client as MongoClient;
use reqwest::Client as HttpClient;
use fantoccini::{
    elements::Element, 
    wd::Locator, 
    Client as WebDriverClient, 
    ClientBuilder
};
use serde_json::{Map, json};
use bytes::Bytes;
use tracing::warn;
use url::Url;
use http::StatusCode;
use anyhow::{Result, bail};

use crate::documents::SpiderOutput;

const UA: &str = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:141.0) Gecko/20100101 Firefox/141.0";
const MAX_W: u32 = 256;
const MAX_H: u32 = 512;

pub struct Spider<'a> {
    mongo: MongoClient,
    http: HttpClient,
    web_driver: WebDriverClient,
    image_root: &'a str
} 

pub struct InitParams<'a> {
    pub mongo_uri: &'a str, 
    pub web_driver_uri: &'a str,
    pub image_root: &'a str
}

pub struct CrawlParams<'a> {
    pub mongo_db: &'a str,
    pub mongo_cl: &'a str,
    pub target_id: i64,
    pub target_url: &'a str
}

struct CrawlOneOutput {
    images: Vec<String>,
    videos: Vec<String>,
    links: Vec<String>
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

        if !fs::try_exists(params.image_root).await? {
            fs::create_dir_all(params.image_root).await?;
        }

        Ok(Self {
            mongo: mongo_client,
            http: http_client,
            web_driver: web_driver_client,
            image_root: params.image_root
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

    async fn extract_video(iframe: Element) -> Result<(String, String)> {
        let Some(src) = iframe.attr("src").await? else {
            bail!("could not find iframe src");            
        };

        if !src.starts_with("https://www.youtube.com/embed/") 
            && !src.starts_with("https://www.youtube-nocookie.com/embed/") 
        {
            bail!("could not find video src");
        }

        let url = Url::parse(&src)?;
        let Some(path) = url.path_segments() else {
            bail!("could not find path segments: {src}");
        };
        let Some(id) = path.skip(1).next() else {
            bail!("could not find video id segment");
        };

        Ok((src, id.to_string()))
    }

    async fn extract_link(a: Element, current_url: &Url) -> Result<String> {
        let Some(mut href) = a.attr("href").await? else {
            bail!("could not find a href");
        };
        if !href.starts_with("https:") && !href.starts_with("http:") {
            let url = current_url.join(&href)?;
            href = url.to_string()
        }

        Ok(href)
    }

    pub async fn crawl(&self, params: CrawlParams<'a>) -> Result<()> {
        let mut crawled_map = HashMap::new();
        let mut output = SpiderOutput{
            mal_id: params.target_id,
            url: params.target_url.to_string(),
            images: vec![],
            videos: vec![],
        };

        let mut q = VecDeque::new();
        q.push_back(params.target_url.to_string());

        loop {
            let Some(next) = q.pop_front() else {
                break;
            };

            if let Some(true) = crawled_map.get(&next) {
                continue;
            }

            let mut out = self.crawl_one(&next).await?;
            output.images.append(&mut out.images);
            output.videos.append(&mut out.videos);
            q.extend(out.links);
            crawled_map.insert(next, true);
        }

        let cl = self.mongo.database(params.mongo_db)
            .collection::<SpiderOutput>(params.mongo_cl);
        cl.insert_one(&output).await?;

        Ok(())
    }

    async fn crawl_one(&self, target_url: &str) -> Result<CrawlOneOutput> 
    {
        let url = Url::parse(target_url)?;

        self.web_driver.goto(target_url).await?;
        self.web_driver.wait().for_url(&url).await?;

        let images = self.scrape_imgs(&url).await?;
        let videos = self.scrape_videos().await?;
        let links = self.scrape_links(&url).await?;

        Ok(CrawlOneOutput {
            images,
            videos,
            links
        })
    }

    async fn scrape_imgs(&self, current_url: &Url) -> Result<Vec<String>> {
        let img_tags = self.web_driver.find_all(Locator::Css("img")).await?;
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
        let img_path = format!("{}/{img_name}", self.image_root);
        if fs::try_exists(&img_path).await? {
            return Ok(None);
        } 

        let url = current_url.join(&src)?;
        let res = self.http.get(url).send().await?;
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

    async fn scrape_videos(&self) -> Result<Vec<String>> {
        let iframes = self.web_driver.find_all(Locator::Css("iframe")).await?;
        let mut output = vec![];

        for iframe in iframes {
            let (_, id) = match Self::extract_video(iframe).await {
                Ok(o) => o,
                Err(e) => {
                    warn!("{e}");
                    continue;
                }
            };

            output.push(id);
        }
        
        Ok(output)
    }

    async fn scrape_links(&self, current_url: &Url) -> Result<Vec<String>> {
        let links = self.web_driver.find_all(Locator::Css("a")).await?;
        let mut output = vec![];

        for link in links {
            let src = match Self::extract_link(link, current_url).await {
                Ok(s) => s,
                Err(e) => {
                    warn!("{e}");
                    continue;
                }
            };

            output.push(src);
        }

        Ok(output)
    }
}
