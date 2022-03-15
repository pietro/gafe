use lambda_runtime::LambdaEvent;
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use std::{collections::HashMap, fmt};
use tracing::{debug, error, info};

mod error;
use error::Result;

mod util;
use crate::util::HTTPResponse;

#[derive(Debug, Deserialize)]
struct Request {
    pub headers: HashMap<String, String>,
    pub uri: String,
}

#[derive(Debug, Serialize)]
struct SuccessResponse {
    pub req_id: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Serialize)]
struct FailureResponse {
    pub req_id: String,
    pub message: String,
}

impl fmt::Display for FailureResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FailureResponse {}

#[tokio::main]
async fn main() -> StdResult<(), lambda_runtime::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("LOG_LEVEL"))
        .without_time()
        .json()
        .init();
    debug!("logger has been set up");

    let func = lambda_runtime::service_fn(handler);

    lambda_runtime::run(func).await?;
    Ok(())
}

/// The handler of the Lambda event. Wraps `fetch` so we can inject the req_id in all responses.
async fn handler(event: LambdaEvent<Request>) -> StdResult<SuccessResponse, FailureResponse> {
    let (req, ctx) = event.into_parts();

    debug!("lambda event request: {:#?}", req);
    debug!("lambda event context: {:#?}", ctx);

    match fetch(req).await {
        Ok(response) => {
            debug!("success");
            Ok(SuccessResponse {
                req_id: ctx.request_id,
                status: response.status,
                headers: response.headers,
                body: response.body,
            })
        }
        Err(e) => {
            error!("something went wrong");
            Err(FailureResponse {
                req_id: ctx.request_id,
                message: e.to_string(),
            })
        }
    }
}

/// Parse the request and get the response.
async fn fetch(request: Request) -> Result<HTTPResponse> {
    let uri_fut = util::parse_uri(&request.uri);
    let headers_fut = util::parse_headers(&request.headers);

    let (uri, headers) = futures_util::try_join!(uri_fut, headers_fut)?;

    info!("fetching uri='{:#?}', headers='{:#?}'", uri, headers);

    let response = util::fetch_uri(headers, uri).await?;

    Ok(response)
}
