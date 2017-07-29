use std::collections::HashMap;

extern crate regex;

const HEADER_REGEX: &str = r"^([A-z\-]+): (.+)$";
const METHOD_REGEX: &str = r"^(GET|POST) (/.*) (HTTPS?)/(\d(?:.\d)*)$";

pub struct HTTPRequest {
    pub method: String,
    pub path: String,
    pub protocol: String,
    pub version: String,
    pub headers: HashMap<String, String> // key, value
}

impl HTTPRequest {
    pub fn parse(read: &str) -> HTTPRequest {
        let mut parsed_headers: HashMap<String, String> = HashMap::new();
        let mut on_method = true;

        let mut method = "GET".to_string(); // default get
        let mut path = "/".to_string(); // default `/`
        let mut protocol = "HTTP".to_string(); // default HTTP
        let mut version = "1.1".to_string(); // default 1.1

        for line in read.split("\r\n") {
            if on_method {
                let re = regex::Regex::new(&METHOD_REGEX).unwrap();
                if re.is_match(line) {
                    let iter = re.captures_iter(line);
                    for matched in iter {
                        // should only run once
                        method = matched[1].to_string();
                        path = matched[2].to_string();
                        protocol = matched[3].to_string();
                        version = matched[4].to_string();
                    }
                }

                on_method = false;
                continue;
            }
            let re = regex::Regex::new(&HEADER_REGEX).unwrap();
            if re.is_match(line) {
                let iter = re.captures_iter(line);
                for matched in iter {
                    // should only loop once
                    parsed_headers.insert(matched[1].to_string(), matched[2].to_string()); // I don't know how to use named groups in this regex implementation
                }
            }
        }
        HTTPRequest {
            method: method,
            path: path,
            protocol: protocol,
            version: version,
            headers: parsed_headers
        }
    }
}
#[derive(Clone)]
pub struct HTTPResponse {
    headers: HashMap<String, String>,
    content_type: String, // placed in headers but here for default value
    // key, value
    content: String,
    version: String, // TODO make this of some integer type
    // string for simplicity
    status: u16
}

impl HTTPResponse {
    pub fn new() ->  HTTPResponse{
        HTTPResponse { headers: HashMap::new(), content_type: "text/html".to_string(), content: String::new(), version: String::from("1.1".to_string()), status: 200 }
    }
    pub fn content(&mut self, content: String) -> &mut HTTPResponse {
        self.headers.insert("content-length".to_string(), (content.len() as u64).to_string());
        self.content = content;
        self
    }
    pub fn version(&mut self, version: String) -> &mut HTTPResponse {
        self.version = version;
        self
    }
    pub fn header(&mut self, key: String, value: String) -> &mut HTTPResponse {
        self.headers.insert(key.to_lowercase(), value);
        self
    }
    pub fn content_type(&mut self, ctype: String) -> &mut HTTPResponse {
        self.content_type = ctype;
        self
    }
    pub fn status(&mut self, status: u16) -> &mut HTTPResponse {
        self.status = status;
        self
    }
    pub fn build(&self) -> String {
        let mut response = String::new();
        // TODO allow for editing of the status message
        response.push_str(&format!("HTTP/{version} {status} OK", version = self.version, status = self.status)[..]);
        response.push_str(&format!("\r\ncontent-type: {}", self.content_type)); //TODO place this in `headers: HashMap`
        for (key, value) in self.headers.clone() {
            response.push_str(&format!("\r\n{key}: {value}", key = key, value = value)[..]);
        }
        response.push_str(&format!("\r\n\r\n{content}", content = self.content)[..]);
        response
    }
}

#[derive(Clone)]
pub enum Location {
    RProxy(String),
    // takes the adress as a string
    Document(String) // takes the location of the document_root
}
