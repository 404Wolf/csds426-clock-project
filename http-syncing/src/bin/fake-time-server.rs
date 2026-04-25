use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tiny_http::{Header, Response, Server};

fn fake_date(offset_s: f64) -> String {
    httpdate::fmt_http_date(SystemTime::now() + Duration::from_secs_f64(offset_s))
}

fn handle(request: tiny_http::Request) {
    let uri: http::Uri = request
        .url()
        .parse()
        .unwrap_or_else(|_| "/".parse().unwrap());
    let offset_s: f64 = uri.path().trim_start_matches('/').parse().unwrap_or(0.0);
    let date = fake_date(offset_s);

    if let Some(addr) = request.remote_addr() {
        eprintln!("{addr} {} -> {date}", uri.path());
    }

    let response = Response::empty(200)
        .with_header(Header::from_bytes("Date", date.as_bytes()).unwrap())
        .with_header(Header::from_bytes("Content-Type", "text/html").unwrap());

    let _ = request.respond(response);
}

fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    let server = Arc::new(Server::http(format!("0.0.0.0:{port}")).unwrap());
    eprintln!("Serving on 0.0.0.0:{port}");

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(10)
        .build()
        .unwrap();

    loop {
        match server.recv() {
            Ok(request) => pool.spawn(move || handle(request)),
            Err(_) => break,
        }
    }
}
