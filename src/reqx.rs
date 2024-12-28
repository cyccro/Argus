use std::fmt::Display;

use http::{HeaderMap, HeaderValue, Method};
use reqwest::{Body, Client, Proxy, RequestBuilder};

#[derive(Debug)]
pub struct Reqx {
    client: Client,
}
#[derive(Debug)]
pub enum ReqxAuth {
    None,
    Basic {
        password: Option<String>,
        username: String,
    },
    Bearer(String),
}
pub struct ReqxData {
    pub headers: HeaderMap<HeaderValue>,
    pub authentication: ReqxAuth,
    pub body: Option<Body>,
    pub form: Option<std::collections::HashMap<String, String>>,
}
impl Default for ReqxData {
    fn default() -> Self {
        Self {
            headers: HeaderMap::default(),
            authentication: ReqxAuth::None,
            body: None,
            form: None,
        }
    }
}
impl Default for Reqx {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}
impl Reqx {
    pub fn new(proxy: Option<Proxy>) -> Self {
        if let Some(proxy) = proxy {
            Self {
                client: Client::builder().proxy(proxy).build().unwrap(),
            }
        } else {
            Self::default()
        }
    }
    pub fn fetch(&self, url: &str, method: http::Method, data: Option<ReqxData>) -> RequestBuilder {
        let req = self.client.request(method.clone(), url);
        if !matches!(method, Method::GET) {
            req
        } else {
            Self::handle_request(req, data.unwrap_or_default())
        }
    }
    fn handle_request(mut req: RequestBuilder, data: ReqxData) -> RequestBuilder {
        req = req.headers(data.headers);
        if let Some(body) = data.body {
            req = req.body(body);
        }
        match data.authentication {
            ReqxAuth::None => {}
            ReqxAuth::Basic { password, username } => req = req.basic_auth(username, password),
            ReqxAuth::Bearer(tk) => req = req.bearer_auth(tk),
        }
        if let Some(form) = data.form {
            req = req.form(&form);
        }
        req
    }
}
