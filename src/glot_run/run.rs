use std::io;
use std::fmt;
use std::time;
use crate::glot_run::api;


#[derive(Debug, serde::Serialize)]
pub struct RunRequest {
    pub image: String,
    pub payload: RunRequestPayload,
}


#[derive(Debug, serde::Serialize)]
pub struct RunRequestPayload {
    pub language: String,
    pub files: Vec<File>,
    pub stdin: Option<String>,
    pub command: Option<String>,
}


#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct File {
    pub name: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RunResult {
    pub stdout: String,
    pub stderr: String,
    pub error: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub base_url: String,
    pub access_token: String,
}

impl Config {
    pub fn run_url(&self) -> String {
        format!("{}/run", self.base_url.trim_end_matches('/'))
    }
}

pub fn run(config: &Config, run_request: RunRequest) -> Result<RunResult, Error> {
    let body = serde_json::to_vec(&run_request)
        .map_err(Error::SerializeRequest)?;

    let response = ureq::post(&config.run_url())
        .set("X-Access-Token", &config.access_token)
        .set("Content-Type", "application/json")
        .timeout(time::Duration::from_secs(300))
        .send_bytes(&body);

    let response = check_response(response)?;

    response.into_json_deserialize()
        .map_err(Error::DeserializeResponse)
}

fn check_response(response: ureq::Response) -> Result<ureq::Response, Error> {
    if !response.ok() {
        if response.synthetic() {
            let err = response.into_synthetic_error()
                .ok_or(Error::EmptySynthetic())?;

            Err(Error::Request(err))
        } else {
            let status_code = response.status();
            let error_body: api::ErrorBody = response.into_json_deserialize()
                .map_err(Error::DeserializeErrorResponse)?;

            Err(Error::ResponseNotOk(api::ErrorResponse{
                status_code,
                body: error_body,
            }))
        }
    } else {
        Ok(response)
    }
}

pub enum Error {
    SerializeRequest(serde_json::Error),
    Request(ureq::Error),
    DeserializeResponse(io::Error),
    DeserializeErrorResponse(io::Error),
    EmptySynthetic(),
    ResponseNotOk(api::ErrorResponse),
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SerializeRequest(err) => {
                write!(f, "Failed to serialize request body: {}", err)
            }

            Error::Request(err) => {
                write!(f, "Request error: {}", err)
            }

            Error::DeserializeResponse(err) => {
                write!(f, "Failed to deserialize response body: {}", err)
            }

            Error::DeserializeErrorResponse(err) => {
                write!(f, "Failed to deserialize error response body: {}", err)
            }

            Error::EmptySynthetic() => {
                write!(f, "Expected synthetic error, but there was none (programming error)")
            }

            Error::ResponseNotOk(err) => {
                write!(f, "Response not ok: {}", err.body.message)
            }
        }
    }
}

