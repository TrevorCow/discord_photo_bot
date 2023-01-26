use std::io::{Cursor, Empty, Read};
use std::sync::{Arc, Mutex};
use std::{io, thread};
use std::fs::File;
use std::path::Path;
use ascii::AsciiString;
use serde::de::Unexpected::Str;
use tiny_http::{Header, Request, Response, ResponseBox, Server, StatusCode};

pub struct PhotoWebserver {
    serving_page_src: Arc<Mutex<Option<String>>>,
}

impl PhotoWebserver {
    pub fn new() -> Self {
        let serving_page_src = Arc::new(Mutex::new(None));
        PhotoWebserver {
            serving_page_src,
        }
    }

    pub fn update_serving_page_src(&mut self, new_serving_page_src: String) {
        let mut spc = self.serving_page_src.lock().unwrap();
        let _ = spc.replace(new_serving_page_src);
    }

    pub fn spawn_server(&self, port: u16) {
        let mut server_addr = "0.0.0.0:".to_string();
        server_addr.push_str(&port.to_string());

        let server = Server::http(server_addr).unwrap();
        let serving_page_src = self.serving_page_src.clone();

        thread::spawn(move || {
            println!("Server started!");
            loop {
                let request = match server.recv() {
                    Ok(request) => { request }
                    Err(err) => {
                        eprintln!("Error in request: {}", err);
                        break;
                    }
                };

                let url = request.url();

                let mut response;
                if url == "/" {
                    response = Self::handle_gallery_request(serving_page_src.clone()).boxed();
                } else {
                    let file_path = String::from(".") + url;
                    let requested_path = Path::new(&file_path);
                    let requested_file_result = File::open(requested_path);
                    if let Ok(requested_file) = requested_file_result {
                        response = Response::from_file(requested_file).boxed();
                        response.add_header(Header {
                            field: "Content-Type".parse().unwrap(),
                            value: AsciiString::from_ascii(get_content_type(requested_path)).unwrap(),
                        });
                    } else {
                        eprintln!("Someone requested invalid file path: `{}` reason: `{}`", url, requested_file_result.err().unwrap());
                        response = Response::empty(StatusCode(301)).boxed();
                        response.add_header(Header::from_bytes(&b"Location"[..], &b"/"[..]).unwrap());
                    }
                }


                request.respond(response).expect("TODO: panic message");
            }
        });
    }

    fn handle_gallery_request(serving_page_src: Arc<Mutex<Option<String>>>) -> Response<Cursor<Vec<u8>>> {
        if let Some(last_built) = serving_page_src.lock().unwrap().as_ref() {
            let data = last_built.clone();
            let data_len = data.len();

            Response::new(
                StatusCode(200),
                vec![
                    Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=UTF-8"[..])
                        .unwrap(),
                ],
                Cursor::new(data.into_bytes()),
                Some(data_len),
                None,
            )
        } else {
            Response::from_string("No built website yet!")
        }
    }
}

fn get_content_type(path: &Path) -> &'static str {
    let extension = match path.extension() {
        None => return "text/plain",
        Some(e) => e,
    };

    match extension.to_str().unwrap() {
        "gif" => "image/gif",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "htm" => "text/html; charset=utf8",
        "html" => "text/html; charset=utf8",
        "txt" => "text/plain; charset=utf8",
        "css" => "text/css; charset=utf8",
        "js" => "text/javascript; charset=utf8",
        _ => "text/plain; charset=utf8",
    }
}