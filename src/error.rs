use reqwest::StatusCode;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    StdError(std::io::Error),
    RequestError(reqwest::Error),
    DeError(quick_xml::DeError),
    ResponseError(StatusCode),
    Utf8Error(std::str::Utf8Error),
    UrlError(url::ParseError),
    InvalidResponse,
    InvalidData(String),
}

impl Error {
    pub fn is_std_err(&self) -> bool {
        matches!(self, Self::StdError(_))
    }

    pub fn is_request_err(&self) -> bool {
        matches!(self, Self::RequestError(_))
    }
    pub fn is_de_err(&self) -> bool {
        matches!(self, Self::DeError(_))
    }
    pub fn is_response_err(&self) -> bool {
        matches!(self, Self::ResponseError(_))
    }

    pub fn is_invalid_utf8_err(&self) -> bool {
        matches!(self, Self::Utf8Error(_))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::StdError(value)
    }
}
impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::RequestError(value)
    }
}
impl From<quick_xml::DeError> for Error {
    fn from(value: quick_xml::DeError) -> Self {
        Self::DeError(value)
    }
}
impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8Error(value)
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Self::UrlError(value)
    }
}

impl From<StatusCode> for Error {
    fn from(value: StatusCode) -> Self {
        Self::ResponseError(value)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StdError(arg0) => arg0.fmt(f),
            Self::RequestError(arg0) => arg0.fmt(f),
            Self::DeError(arg0) => arg0.fmt(f),
            Self::ResponseError(arg) => arg.fmt(f),
            Self::Utf8Error(arg) => arg.fmt(f),
            Self::UrlError(parse_error) => parse_error.fmt(f),
            Self::InvalidResponse => f.write_str("InvalidResponse"),
            Self::InvalidData(data) => f.write_str(&format!("Invalid data: {}", data)),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StdError(arg0) => arg0.fmt(f),
            Self::RequestError(arg0) => arg0.fmt(f),
            Self::DeError(arg0) => arg0.fmt(f),
            Self::ResponseError(arg) => arg.fmt(f),
            Self::Utf8Error(arg) => arg.fmt(f),
            Self::UrlError(arg) => arg.fmt(f),
            Self::InvalidResponse => f.write_str("Invalid response received"),
            Self::InvalidData(data) => f.write_str(data),
        }
    }
}
impl std::error::Error for Error {}
