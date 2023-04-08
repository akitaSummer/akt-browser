use super::url::{ParseError, Url};
use log::{error, info};
use num_derive::FromPrimitive;
use std::fs;
use std::{collections::HashMap, fmt, str::FromStr};
use thiserror::Error;

#[derive(Debug, PartialEq, FromPrimitive)]
pub enum HTTPStatus {
    OK = 200,
}

#[derive(Debug, PartialEq)]
pub enum ResponseType {
    Basic,
    CORS,
    Default,
    Error,
}

impl fmt::Display for ResponseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResponseType::Basic => f.write_str("basic"),
            ResponseType::CORS => f.write_str("cors"),
            ResponseType::Default => f.write_str("default"),
            ResponseType::Error => f.write_str("error"),
        }
    }
}

impl FromStr for ResponseType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "basic" => Ok(ResponseType::Basic),
            "cors" => Ok(ResponseType::CORS),
            "default" => Ok(ResponseType::Default),
            "error" => Ok(ResponseType::Error),
            _ => Err("invalid response type"),
        }
    }
}

pub type HeaderMap = HashMap<String, String>;

#[derive(Debug, PartialEq)]
pub struct Request {
    pub url: String,
}

impl Request {
    pub fn new(url: String) -> Self {
        Request { url: url }
    }
}

#[derive(Debug, PartialEq)]
pub struct Response {
    pub rtype: ResponseType,
    pub url: Url,
    pub status: HTTPStatus,
    pub headers: HeaderMap,
    pub data: Vec<u8>,
}

#[derive(Error, Debug, PartialEq)]
pub enum FetchError {
    #[error("failed to fetch because of something")]
    NetworkError { response: Option<Response> },

    #[error("failed to fetch because given url is invalid")]
    URLParseError {
        error: ParseError,
        response: Option<Response>,
    },

    #[error("failed to fetch because scheme {scheme:?} is not supported")]
    URLSchemeUnsupportedError {
        scheme: String,
        response: Option<Response>,
    },
}

pub fn fetch(request: Request) -> Result<Response, FetchError> {
    match Url::parse(request.url.as_str()) {
        Ok(u) => match u.scheme() {
            "file" => {
                info!("[file:] local resource at {} is requested.", u.path());
                match fs::read(u.path()) {
                    Ok(content) => Ok(Response {
                        url: u,
                        status: HTTPStatus::OK,
                        rtype: ResponseType::Basic,
                        headers: HeaderMap::new(),
                        data: content,
                    }),
                    Err(_e) => Err(FetchError::NetworkError { response: None }),
                }
            }
            // 不实现http(s)
            "http" | "https" => {
                info!(
                    "[http(s):] remote resource at {} is requested.",
                    u.to_string()
                );
                Err(FetchError::URLSchemeUnsupportedError {
                    scheme: "http(s)".to_string(),
                    response: None,
                })
            }
            unsupported_scheme => Err(FetchError::URLSchemeUnsupportedError {
                scheme: unsupported_scheme.to_string(),
                response: None,
            }),
        },
        // 解析出错
        Err(e) => Err(FetchError::URLParseError {
            error: e,
            response: None,
        }),
    }
}
