use hyper::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Uri,
};
use hyper_tls::HttpsConnector;
use std::collections::HashMap;
use tracing::{debug, error};

use crate::error::{Error, Result};

pub struct HTTPResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Parse Map of header string into a hyper HeaderMap.
pub async fn parse_headers(headers: &HashMap<String, String>) -> Result<HeaderMap> {
    let mut header_map = HeaderMap::with_capacity(headers.len());
    for (name, value) in headers {
        let header_name: HeaderName = name.parse()?;
        let value = HeaderValue::try_from(value.trim_start())?;
        header_map.insert(header_name, value);
    }
    debug!("request_headers: {:?}: ", header_map);
    Ok(header_map)
}

/// Parse and validate the URI.
pub async fn parse_uri(req_uri: &str) -> Result<Uri> {
    let uri = Uri::try_from(req_uri)?;

    match uri.scheme() {
        Some(_) => {
            debug!("uri: {:?}", uri);
            Ok(uri)
        }
        None => {
            error!("empty scheme for  URI: {}", uri);
            Err(Error {
                message: String::from("empty scheme"),
            })
        }
    }
}

/// Convert response headers to a Map of string keys to String Values
async fn stringfy_headers(header_map: &HeaderMap) -> Result<HashMap<String, String>> {
    let mut headers = HashMap::with_capacity(header_map.len());

    for (k, v) in header_map {
        let header_name = k.to_string();
        let header_value = v.to_str()?;
        headers.insert(header_name, String::from(header_value));
    }

    Ok(headers)
}

/// Get the bytes buffer of the request body and base64 encode it
async fn get_body(body: hyper::Body) -> Result<String> {
    let bytes = hyper::body::to_bytes(body).await?;
    let body = base64::encode(&bytes);
    Ok(body)
}

/// Go get it!
pub async fn fetch_uri(req_headers: HeaderMap, uri: Uri) -> Result<HTTPResponse> {
    let https = HttpsConnector::new();

    let client = Client::builder().build::<_, hyper::Body>(https);

    let (mut parts, body) = hyper::Request::default().into_parts();

    parts.method = hyper::Method::GET;
    parts.headers = req_headers;
    parts.uri = uri;

    let request = hyper::Request::from_parts(parts, body);

    let response = client.request(request).await?;

    let (head, body) = response.into_parts();

    let status = head.status.as_u16();

    let headers_fut = stringfy_headers(&head.headers);

    let bytes_fut = get_body(body);

    let (headers, body) = futures_util::try_join!(headers_fut, bytes_fut)?;

    Ok(HTTPResponse {
        status,
        headers,
        body,
    })
}

#[cfg(test)]
mod tests {
    mod parse_uri {
        use crate::util::parse_uri;
        use hyper::Uri;

        #[tokio::test]
        async fn http() {
            let uri = "http://example.org";
            let actual = parse_uri(uri).await.unwrap();

            let expected = Uri::from_static("http://example.org/");
            assert_eq!(actual, expected);
        }

        #[tokio::test]
        async fn https() {
            let uri = "https://example.org";
            let actual = parse_uri(uri).await.unwrap();
            let expected = Uri::from_static("https://example.org/");
            assert_eq!(actual, expected);
        }

        #[tokio::test]
        async fn no_scheme() {
            let uri = "example.org";
            assert!(parse_uri(uri).await.is_err());
        }

        #[tokio::test]
        async fn invalid_scheme() {
            let uri = "file:///etc/passwd";
            assert!(parse_uri(uri).await.is_err());
        }

        #[tokio::test]
        async fn mail_to() {
            let uri = "mailto:address@example.org";
            assert!(parse_uri(uri).await.is_err());
        }
    }

    mod parse_headers {
        use crate::util::parse_headers;
        use hyper::header::{HeaderMap, HeaderName, HeaderValue};
        use std::collections::HashMap;

        #[tokio::test]
        async fn user_agent() {
            let ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.4844.51 Safari/537.36";
            let mut headers = HashMap::<String, String>::new();
            headers.insert(String::from("User-Agent"), String::from(ua));

            let actual = parse_headers(&headers).await.unwrap();

            let mut map = HeaderMap::new();
            map.insert(hyper::header::USER_AGENT, ua.parse().unwrap());
            let expected = map;

            assert_eq!(actual, expected);
        }

        #[tokio::test]
        async fn multiple() {
            let x_foo = "X-FoO";
            let foo = "foo";
            let x_bar = "x-bar";
            let bar = "BaR";

            let mut headers = HashMap::<String, String>::new();
            headers.insert(x_foo.to_string(), foo.to_string());
            headers.insert(x_bar.to_string(), bar.to_string());

            let actual = parse_headers(&headers).await.unwrap();

            let mut map = HeaderMap::new();
            map.insert(
                HeaderName::from_static(x_bar),
                HeaderValue::from_static(bar),
            );
            map.insert(
                HeaderName::from_static("x-foo"),
                HeaderValue::from_static(foo),
            );
            let expected = map;

            assert_eq!(actual, expected);
        }

        #[tokio::test]
        async fn invalid_header_name() {
            let mut headers = HashMap::<String, String>::new();
            headers.insert(String::from("Whaaaaa?"), String::from("value"));
            assert!(parse_headers(&headers).await.is_err());
        }

        #[tokio::test]
        async fn empty_header_name() {
            let mut headers = HashMap::<String, String>::new();
            headers.insert(String::from(""), String::from("value"));
            assert!(parse_headers(&headers).await.is_err());
        }
    }
}
