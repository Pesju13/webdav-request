mod client;
mod error;
mod method;
mod reader;
mod res;

pub use method::Method;
pub use reqwest::header;
pub use reqwest::{Body, IntoUrl, Request, RequestBuilder, Response, StatusCode, Url};

pub use crate::client::*;
pub use crate::reader::LazyResponseReader;
pub use crate::res::*;
pub use error::*;
pub use quick_xml::DeError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}
impl Range {
    pub fn new(start: usize, end: usize) -> Range {
        Self { start, end }
    }
}
