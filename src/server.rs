use std::thread;
use std::thread::JoinHandle;
use std::net::*;
use std::io::*;
use protocol::{HTTPRequest, Location, HTTPResponse};
use std::fs::File;
use std::path::Path;
use std::ffi::OsStr;
use std::net::TcpStream;

pub struct Server {
    // pub name: String,
    pub host: String,
    pub port: u16,
    pub location: Location
}

impl Clone for Server {
    fn clone(&self) -> Self {
        Server {
            host: self.host.clone(),
            port: self.port,
            location: self.location.clone(),
        }
    }
}

impl Server {
    pub fn start(self) -> JoinHandle<()> {
        let server_thread = thread::spawn(move || {
            self.run();
        });
        // I want to return server-thread
        server_thread
    }
    fn run(&self) {
        let mut bind_addr = String::from("127.0.0.1:");
        bind_addr.push_str(&self.port.to_string());
        let listener = TcpListener::bind(bind_addr).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let clone = self.clone();
            thread::spawn(move || clone.handle_connection(stream));
        }
    }
    fn handle_connection(&self, mut stream: TcpStream) {
        let mut buff = [0; 1024];
        stream.read(&mut buff).expect("Failed to read input");
        let read = String::from_utf8_lossy(&buff);
        let request = HTTPRequest::parse(&read);
        if let Location::Document(ref root) = self.location {
            let mut extra = "";
            if request.path.ends_with("/") {
                extra = "index.html";
            }
            let location = format!("{}{}{}", root, request.path, extra); //TODO possible directory traversal
            let mut response = HTTPResponse::new();
            let clone = request.path.clone();
            if Server::is_image(request.path) {
                response.content_type(format!("img/{}", Server::get_extension(&clone)));
            }
            let mut content = String::new();
            match File::open(location) {
                Ok(file) => { let _ = BufReader::new(file).read_to_string(&mut content); }
                Err(error) => {
                    println!("Error: {}", error);
                    content = String::from("Hello! This is nserv, a HTTP Server completely written in Rust. The requested page was unable to be returned, please try a different page or come back later");
                    response.status(404); // give a relevant message next time
                }
            }
            let _ = BufWriter::new(stream).write(
                HTTPResponse::new().content(content).version("1.1".to_string()).build().as_bytes() // status already set (default 200)
            );
        } else if let Location::RProxy(ref address) = self.location {
            let to_send = match TcpStream::connect(address) {
                Ok(ref receiver) => {
                    let _  = BufWriter::new(receiver).write(&buff); // write what was read
                    let mut read_buff = [0;10240]; // max 10 megabytes
                    let _ = BufReader::new(receiver).read(&mut read_buff); // read what was returned
                    String::from_utf8_lossy(&read_buff).to_string() // send to client what was returned
                }
                Err(error) => {
                    println!("Error: {}", error);
                    "Experienced an error while connecting to the end server".to_string()
                }
            };
            let _ = BufWriter::new(stream).write(&to_send.as_bytes());
        }
    }
    fn get_extension(file_name: &str) -> String {
        String::from(Path::new(file_name)
            .extension()
            .and_then(OsStr::to_str).unwrap_or(""))
    }
    fn is_image(file_name: String) -> bool {
        let images = vec!["ico", "png", "jpg", "gif"];
        images.contains(&&Server::get_extension(&file_name)[..])
    }
}
