use std::fmt;

#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

///
pub fn failed_to_parse_json(msg: &str) -> Error {
    Error(format!(
        "[fred_app:01] Failed to parse JSON with error [{}].",
        msg,
    ))
}

///
pub fn failed_http_request(msg: &str) -> Error {
    Error(format!(
        "[fred_api:02]] Http request failed with error [{}].",
        msg,
    ))
}

