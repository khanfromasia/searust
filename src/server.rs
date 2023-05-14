use std::fs::File;
use std::str;
use std::io;

use super::model::*;

use tiny_http::{Server, Response, Header, Method, Request, StatusCode};


fn serve_error(request: Request, status_code: i32, message: &str) -> io::Result<()> {
    request.respond(Response::from_string(format!("{status_code}: {message}")).with_status_code(StatusCode::from(status_code)))
}

fn serve_static_file(request: Request, file_path: &str, content_type: &str) -> io::Result<()> {
    let content_type_header = Header::from_bytes("Content-Type", content_type)
        .expect("That we didn't put any garbage in the headers");

    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("ERROR: could not serve file {file_path}: {err}");
            if err.kind() == io::ErrorKind::NotFound {
                return serve_error(request, 404, "not found");
            }
            return serve_error(request, 500, "internal server error");
        }
    };

    request.respond(Response::from_file(file).with_header(content_type_header))
}

fn search_handler(mut request: Request, model: &InMemoryModel) -> io::Result<()> {
    let mut buf = Vec::new();
    if let Err(err) = request.as_reader().read_to_end(&mut buf) {
        eprintln!("ERROR: could not read the body of the request: {err}");
        return serve_error(request, 500, "internal server error");
    }

    let body = match str::from_utf8(&buf) {
        Ok(body) => body.chars().collect::<Vec<_>>(),
        Err(err) => {
            eprintln!("ERROR: could not interpret body as UTF-8 string: {err}");
            return serve_error(request, 400, "body must be a valid UTF-8 string");
        },
    };
    
    let result = model.search_query(&body);

    let json = match serde_json::to_string(&result.iter().take(20).collect::<Vec<_>>()) {
        Ok(json) => json, 
        Err(err) => {
            eprintln!("ERROR: could not convert search results to JSON: {err}");
            return serve_error(request, 500, "internal server error")
        },
    };

    let content_type_header = Header::from_bytes("Content-Type", "application/json")
        .expect("That we didn't put any garbage in the headers");

    request.respond(Response::from_string(&json).with_header(content_type_header))
}

fn serve_request(request: Request, model: &InMemoryModel) -> io::Result<()> {
    println!("INFO: request received: {:?} {:?}", request.method(), request.url());

    match request.method() {
        Method::Get => {
            match request.url() {
                "/" | "/index.html" => {
                    serve_static_file(request, "index.html", "text/html; charset=utf-8")
                },
                "/index.js" => {
                    serve_static_file(request, "index.js", "text/javascript; charset=utf-8")
                },
                _ => {
                    serve_error(request, 404, "not found")
                }
            }
        },
        Method::Post => {
            match request.url() {
                "/api/search" => {
                    search_handler(request, model)
                }, 
                _ => {
                    serve_error(request, 404, "not found")
                }
            }
        },
        _ => {
            serve_error(request, 404, "not found")
        }
    }
}  



pub fn start(address: &str, model: &InMemoryModel) -> Result<(), ()> {
    let server = Server::http(&address).map_err(|err| {
        eprintln!("ERROR: could not start HTTP server at {address}: {err}");
    })?;

    println!("INFO: listening at http://{address}/");

    for request in server.incoming_requests() {
        serve_request(request, model).map_err(|err| {
            eprintln!("ERROR: could not serve the response: {err}");
        }).ok();
    }

    eprintln!("ERROR: the server socket has shutdown");
    Err(())
}