use crate::{RequestBuilder, Response};
use std::{
    future::Future,
    io::{self},
    pin::{pin, Pin},
    task::Poll,
};
type ResponseStream = Box<dyn Unpin + Future<Output = reqwest::Result<Response>>>;

pub struct LazyResponseReader {
    request: Option<RequestBuilder>,
    stream: Option<ResponseStream>,
    reader: Option<ResponseReader>,
}

impl From<RequestBuilder> for LazyResponseReader {
    fn from(value: RequestBuilder) -> Self {
        Self::new(value)
    }
}
impl From<ResponseStream> for LazyResponseReader {
    fn from(value: ResponseStream) -> Self {
        Self {
            request: None,
            stream: Some(value),
            reader: None,
        }
    }
}
impl LazyResponseReader {
    pub fn new(builder: RequestBuilder) -> Self {
        Self {
            request: Some(builder),
            stream: None,
            reader: None,
        }
    }
}
impl Unpin for LazyResponseReader {}

macro_rules! ready {
    ($n:expr) => {
        if ($n == 0) {
            return Poll::Ready(Ok(()));
        }
    };
    ($v1:expr, $v2:expr) => {
        if ($v1 == $v2) {
            return Poll::Ready(Ok(()));
        }
    };
}

impl tokio::io::AsyncRead for LazyResponseReader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        ready!(buf.remaining());

        let this = self.get_mut();

        if this.request.is_some() {
            let request = this.request.take().unwrap();
            this.stream = Some(Box::new(request.send()));
        }
        if let Some(send) = &mut this.stream {
            match Future::poll(Pin::new(send), cx) {
                Poll::Ready(data) => match data {
                    Ok(response) => {
                        if !response.status().is_success() {
                            return Poll::Ready(Err(io::Error::other(
                                response.status().to_string(),
                            )));
                        }
                        this.stream = None;
                        this.reader = Some(ResponseReader::new(response))
                    }
                    Err(e) => return Poll::Ready(Err(io::Error::other(e.to_string()))),
                },
                Poll::Pending => return Poll::Pending,
            }
        }

        if let Some(val) = &mut this.reader {
            return Pin::new(val).poll_read(cx, buf);
        }

        Poll::Ready(Ok(()))
    }
}

#[derive(Default)]
pub struct ResponseReader {
    inner: Option<Response>,
    buf: Vec<u8>,
}

impl std::fmt::Debug for ResponseReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            Some(res) => res.fmt(f),
            _ => f.write_str("None"),
        }
    }
}
impl From<Response> for ResponseReader {
    fn from(value: Response) -> Self {
        Self::new(value)
    }
}
impl ResponseReader {
    pub fn new(response: Response) -> ResponseReader {
        Self {
            inner: Some(response),
            ..Default::default()
        }
    }
}

impl tokio::io::AsyncRead for ResponseReader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        ready!(buf.remaining());
        let this = self.get_mut();
        if let Some(res) = &mut this.inner {
            loop {
                let chunk = res.chunk();
                match Future::poll(pin!(chunk), cx) {
                    Poll::Ready(Ok(bytes)) => {
                        if let Some(bytes) = bytes {
                            this.buf.extend_from_slice(&bytes.slice(0..bytes.len()));
                            let remain = buf.remaining();
                            if this.buf.len() >= remain {
                                buf.put_slice(&this.buf[..remain]);
                                this.buf = this.buf[remain..].to_owned();
                                return Poll::Ready(Ok(()));
                            }
                        } else {
                            let remain = buf.remaining();
                            if this.buf.len() >= remain {
                                buf.put_slice(&this.buf[..remain]);
                                this.buf = this.buf[remain..].to_owned();
                            } else {
                                buf.put_slice(&this.buf);
                                this.buf = Vec::new();
                            }
                            return Poll::Ready(Ok(()));
                        }
                    }
                    Poll::Ready(Err(err)) => {
                        return Poll::Ready(Err(io::Error::other(err.to_string())));
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }
        } else {
            Poll::Pending
        }
    }
}

impl Unpin for ResponseReader {}
