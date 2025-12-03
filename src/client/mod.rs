mod inner;
use std::sync::Arc;
pub(crate) mod webdav_client;
use crate::method::Method;
use crate::multistatus::MultiStatus;
use crate::reader::LazyResponseReader;
use crate::DavItem;
use crate::{header::HeaderMap, Body};
use inner::InnerClient;
use reqwest::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::{IntoUrl, Response, Url};
const DEPTH: HeaderName = HeaderName::from_static("depth");
const DEPTH_ONE: HeaderValue = HeaderValue::from_static("1");
const RANGE: HeaderName = HeaderName::from_static("range");
const APPLICATION_XML: HeaderValue = HeaderValue::from_static("application/xml");

macro_rules! header_value {
    ($arg:expr) => {
        reqwest::header::HeaderValue::from_bytes($arg.as_bytes()).unwrap()
    };
}

const ALL_DROP: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
    <D:propfind xmlns:D="DAV:">
        <D:allprop/>
    </D:propfind>
"#;

#[derive(Default, Clone)]
pub struct DavClient {
    inner: Arc<InnerClient>,
}
unsafe impl Send for DavClient {}

unsafe impl Sync for DavClient {}

impl DavClient {
    pub fn new(username: &str, password: &str) -> crate::Result<Self> {
        Ok(Self {
            inner: Arc::new(InnerClient::new(username, password)?),
        })
    }
    pub fn request(&self, method: Method, url: Url) -> WevDAVRequestBuilder {
        WevDAVRequestBuilder::new(self.inner.clone(), url, method)
    }

    /// Initiates a GET request to retrieve a resource from the WebDAV server.
    #[inline(always)]
    pub fn get(&self, url: impl IntoUrl) -> crate::Result<WevDAVRequestBuilder> {
        Ok(self.request(Method::GET, url.into_url()?))
    }

    /// Initiates a PUT request to upload or update a resource on the WebDAV server.
    #[inline(always)]
    pub fn put(&self, url: impl IntoUrl) -> crate::Result<WevDAVRequestBuilder> {
        Ok(self.request(Method::PUT, url.into_url()?))
    }

    /// Lists the contents of a WebDAV collection (directory).
    pub async fn list(&self, url: impl IntoUrl) -> crate::Result<Vec<DavItem>> {
        let url = url.into_url()?;
        let url_str = url.as_str().to_owned();
        let response = self.all_propfind(url).await?;
        if response.status().is_success() {
            let xml = response.text().await?;
            let multi_status = MultiStatus::parse(&xml)?;
            DavItem::parse(multi_status, &url_str)
        } else {
            Err(response.status().into())
        }
    }

    #[inline(always)]
    pub async fn all_propfind(&self, url: impl IntoUrl) -> crate::Result<Response> {
        self.request(Method::PROPFIND, url.into_url()?)
            .header(CONTENT_TYPE, APPLICATION_XML)
            .header(DEPTH, DEPTH_ONE)
            .body(ALL_DROP)
            .send()
            .await
    }
}

pub struct WevDAVRequestBuilder {
    client: Arc<InnerClient>,
    basic_auth: Option<(String, String)>,
    url: Url,
    headers: HeaderMap,
    body: Option<Body>,
    method: Method,
}

impl WevDAVRequestBuilder {
    pub fn new(client: Arc<InnerClient>, url: Url, method: Method) -> Self {
        Self {
            client,
            basic_auth: None,
            headers: HeaderMap::new(),
            url,
            method,
            body: None,
        }
    }
    pub fn basic_auth(self, username: &str, password: &str) -> Self {
        Self {
            basic_auth: Some((username.to_owned(), password.to_owned())),
            ..self
        }
    }
    pub fn body(self, body: impl Into<Body>) -> Self {
        Self {
            body: Some(body.into()),
            ..self
        }
    }
    pub fn range(self, start: usize, end: usize) -> Self {
        self.header(RANGE, header_value!(format!("bytes={}-{}", start, end)))
    }
    pub fn header(mut self, key: HeaderName, val: HeaderValue) -> Self {
        self.headers.insert(key, val);
        self
    }
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers.extend(headers);
        self
    }

    pub fn build(self) -> crate::RequestBuilder {
        let builder = self.client.inner.request(self.method.convert(), self.url);
        let builder = if let Some(body) = self.body {
            builder.body(body)
        } else {
            builder
        };
        if let Some((usr, pass)) = &self.basic_auth {
            builder.basic_auth(usr, Some(pass))
        } else if let Some((usr, psw)) = &self.client.auth {
            builder.basic_auth(usr, Some(psw))
        } else {
            builder
        }
        .headers(self.headers)
    }
    pub fn into_lazy_reader(self) -> LazyResponseReader {
        LazyResponseReader::new(self.build())
    }
    pub async fn send(self) -> crate::Result<Response> {
        self.build().send().await.map_err(Into::into)
    }
}
