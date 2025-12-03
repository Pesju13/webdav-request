use std::sync::Arc;

use crate::client::inner::InnerClient;
use crate::client::ALL_DROP;
use crate::method::Method;
use crate::multistatus::MultiStatus;
use crate::WevDAVRequestBuilder;
use reqwest::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::{IntoUrl, Response};
const DEPTH: HeaderName = HeaderName::from_static("depth");
const DEPTH_ONE: HeaderValue = HeaderValue::from_static("1");
const APPLICATION_XML: HeaderValue = HeaderValue::from_static("application/xml");
#[deprecated = "Use `DavClient` instead"]
#[derive(Default, Clone)]
pub struct WebDAVClient {
    inner: Arc<InnerClient>,
}
#[allow(deprecated)]
unsafe impl Send for WebDAVClient {}

#[allow(deprecated)]
unsafe impl Sync for WebDAVClient {}

macro_rules! into_url {
    ($url:expr) => {
        match $url.into_url() {
            Ok(url) => url,
            Err(e) => panic!("{e}"),
        }
    };
}

#[allow(deprecated)]
impl WebDAVClient {
    pub fn new(username: &str, password: &str) -> Result<Self, reqwest::Error> {
        Ok(Self {
            inner: Arc::new(InnerClient::new(username, password)?),
        })
    }
    pub fn request(&self, method: Method, url: impl IntoUrl) -> WevDAVRequestBuilder {
        WevDAVRequestBuilder::new(self.inner.clone(), into_url!(url), method)
    }

    #[inline(always)]
    pub fn get(&self, url: impl IntoUrl) -> WevDAVRequestBuilder {
        self.request(Method::GET, url)
    }

    #[inline(always)]
    pub fn put(&self, url: impl IntoUrl) -> WevDAVRequestBuilder {
        self.request(Method::PUT, url)
    }

    pub async fn list(
        &self,
        url: impl IntoUrl,
    ) -> Result<crate::res::Collection, crate::error::Error> {
        let response = self.all_propfind(url).await?;
        if response.status().is_success() {
            let xml = response.text().await?;
            let multi_status = MultiStatus::parse(&xml)?;
            Ok(crate::res::Collection::from(multi_status))
        } else {
            Err(crate::error::Error::ResponseError(response.status()))
        }
    }
    #[inline(always)]
    pub async fn all_propfind(&self, url: impl IntoUrl) -> crate::Result<Response> {
        self.request(Method::PROPFIND, url.into_url()?)
            .header(CONTENT_TYPE, APPLICATION_XML.clone())
            .header(DEPTH.clone(), DEPTH_ONE.clone())
            .body(ALL_DROP)
            .send()
            .await
    }
}
