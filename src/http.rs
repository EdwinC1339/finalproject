use std::{fmt::Display, sync::Arc};

#[derive(Debug, Clone)]
pub struct HTTPRequest {
    method: HTTPMethod,
    uri: HTTPUri,
    version: HTTPVersion,
    headers: Vec<HTTPHeader>,
    body: HTTPBody
}

#[derive(Debug, Clone)]
pub enum HTTPMethod {
    Get,
    Post
}

#[derive(Debug, Clone)]
pub struct HTTPUri(Arc<str>);
#[derive(Debug, Clone)]
pub struct HTTPVersion(Arc<str>);

#[derive(Debug, Clone)]
pub struct HTTPHeader {
    name: Arc<str>,
    value: Arc<str>
}

#[derive(Debug, Clone)]
pub struct HTTPBody {
    body: Arc<str>
}

impl HTTPRequest {
    pub fn new<'a>(mut http_lines: impl Iterator<Item = String>) -> Option<Self> {
        let top_line = http_lines.next()?;
        let mut top_line = top_line.split_ascii_whitespace();

        let method = top_line.next()?
            .try_into().ok()?;
        let uri = top_line.next()?
            .into();
        let version = top_line.next()?
            .into();

        let headers = 
            http_lines.flat_map(|line| line.as_str().try_into())
            .collect();

        let body = HTTPBody { body: "".into() };

        Some(HTTPRequest { method, uri, version, headers, body})
    }

    pub fn method(&self) -> &HTTPMethod {
        &self.method
    }

    pub fn uri(&self) -> &HTTPUri {
        &self.uri
    }

    pub fn version(&self) -> &HTTPVersion {
        &self.version
    }

    pub fn headers(&self) -> &[HTTPHeader] {
        &self.headers
    }

    pub fn body(&self) -> &HTTPBody {
        &self.body
    }
}

impl TryFrom<&str> for HTTPMethod {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            _ => Err(())
        }
    }
}

impl From<&str> for HTTPUri {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl HTTPUri {
    pub fn unwrap(&self) -> &str {
        &self.0
    }
}

impl From<&str> for HTTPVersion {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl HTTPBody {
    pub fn new(body: &str) -> Self {
        Self { body: body.into() }
    }

    pub fn unwrap(&self) -> &str {
        &self.body
    }
}

impl From<&str> for HTTPBody {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<Arc<str>> for HTTPBody {
    fn from(value: Arc<str>) -> Self {
        Self { body: value }
    }
}

impl HTTPHeader {
    pub fn new(name: &str, value: &str)  -> Self {
        HTTPHeader {
            name: name.into(),
            value: value.into()
        }
    }
}

impl TryFrom<&str> for HTTPHeader {
    type Error = ();

    fn try_from(from_val: &str) -> Result<Self, Self::Error> {
        let mut it = from_val.split(": ");
        Ok(HTTPHeader{
            name: it.next().ok_or(())?.into(), 
            value: it.next().ok_or(())?.into(),
        })
    }
}

impl Display for HTTPHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

#[derive(Debug, Clone)]
pub struct HTTPResponse {
    status: HTTPStatus,
    headers: Vec<HTTPHeader>,
    body: HTTPBody,
}

impl HTTPResponse {
    pub fn new<'a>(
        status: &str, 
        headers: impl Iterator<Item = &'a String>,
        body: String
    ) -> Option<Self> {
        let length_header = HTTPHeader::new("Content-Length", format!("{}", body.len()).as_str());
        let status  = status.into(); 
        let mut headers: Vec<_> = headers.flat_map(|header| header.
            as_str()
            .try_into()
        )
        .collect();
        headers.push(length_header);
    
        let body = body.as_str().into();
    
        Some(HTTPResponse {
            status, headers, body
        })
    }

    fn status_line(&self) -> String {
        format!("HTTP/1.1 {}\r\n", self.status)
    }

    pub fn to_string(self) -> String {
        let mut out = self.status_line();
        let content = self.body.unwrap();
        for header in self.headers {
            out.push_str(&format!("{}\r\n", header));
        }
        out.push_str("\r\n");
        out.push_str(content);

        out
    }


}

#[derive(Debug, Clone)]
pub struct HTTPStatus(Arc<str>);

impl HTTPStatus {
    pub fn new(value: &str) -> Self {
        Self ( value.into() )
    }
}

impl From<&str> for HTTPStatus {
    fn from(value: &str) -> Self {
        HTTPStatus::new(value)
    }
}

impl Display for HTTPStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for HTTPResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_line = self.status_line();
        write!(f, "{status_line}\r\n")
    }
}