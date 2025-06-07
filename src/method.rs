#[derive(Clone)]
enum Inner {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Propfind,
    Custom(reqwest::Method),
}
#[derive(Clone)]
pub struct Method(Inner);

impl Method {
    pub const GET: Method = Method(Inner::Get);
    pub const POST: Method = Method(Inner::Post);
    pub const PUT: Method = Method(Inner::Put);
    pub const DELETE: Method = Method(Inner::Delete);
    pub const PATCH: Method = Method(Inner::Patch);
    pub const PROPFIND: Method = Method(Inner::Propfind);
    pub(crate) fn convert(self) -> reqwest::Method {
        use reqwest::Method as RMethod;
        match self.0 {
            Inner::Get => RMethod::GET,
            Inner::Post => RMethod::POST,
            Inner::Put => RMethod::PUT,
            Inner::Delete => RMethod::DELETE,
            Inner::Patch => RMethod::PATCH,
            Inner::Propfind => RMethod::from_bytes("PROPFIND".as_bytes()).unwrap(),

            Inner::Custom(method) => method,
        }
    }
    pub fn from_bytes(src: &[u8]) -> Result<Self, String> {
        match reqwest::Method::from_bytes(src) {
            Ok(m) => Ok(Self(Inner::Custom(m))),
            Err(e) => Err(e.to_string()),
        }
    }
}
