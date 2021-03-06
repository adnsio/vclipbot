use crate::dal::response::clip_response::ClipResponse;
use async_trait::async_trait;
use scraper::{Html, Selector};
use std::error::Error;

struct Endpoints;

impl Endpoints {
    fn query(query: &str) -> String {
        format!("yarn-find?text={}", query)
    }
}

#[async_trait]
pub trait YarnApi: Sync + Send {
    async fn query(&self, query: &str) -> Result<Vec<ClipResponse>, Box<dyn Error>>;
}

#[derive(Clone)]
pub struct YarnApiImpl {
    base_url: String,
}

impl YarnApiImpl {
    pub fn new(base_url: String) -> YarnApiImpl {
        YarnApiImpl { base_url }
    }
}

#[async_trait]
impl YarnApi for YarnApiImpl {
    async fn query(&self, query: &str) -> Result<Vec<ClipResponse>, Box<dyn Error>> {
        let endpoint = format!("{}/{}", self.base_url, Endpoints::query(query));
        let response = reqwest::get(&endpoint).await?;
        let html = response.text().await?;
        let document = Html::parse_document(&html);
        let clip_wrap_selector = Selector::parse(".clip-wrap").unwrap();
        let title_selector = Selector::parse(".title").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        let mut ret: Vec<ClipResponse> = Vec::new();

        for clip_wrap in document.select(&clip_wrap_selector) {
            let mut clip_wrap_links = clip_wrap.select(&a_selector);
            let first_link = clip_wrap_links.next().unwrap();
            let first_link_href = first_link.value().attr("href").unwrap();
            let second_link = clip_wrap_links.next().unwrap();

            let clip_title = first_link
                .select(&title_selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>();
            let clip_id = first_link_href.replace("/yarn-clip/", "");
            let clip_text = second_link.text().collect::<String>();

            ret.push(ClipResponse {
                id: clip_id.clone(),
                title: clip_title,
                caption: clip_text,
                mp4_link: format!("https://y.yarn.co/{}_text.mp4", clip_id),
                gif_link: format!("https://y.yarn.co/{}_text.gif", clip_id),
            })
        }
        Ok(ret)
    }
}
