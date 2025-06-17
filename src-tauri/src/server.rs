use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use multer::Multipart;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncWriteExt;

/// Port number the HTTP server listens on.
pub const PORT: u16 = 8080;

/// Maximum allowed upload size in bytes (4 GiB).
pub const MAX_UPLOAD_SIZE: u64 = 4 * 1024 * 1024 * 1024;

/// Returns the port the server listens on.
pub fn port() -> u16 {
    PORT
}

async fn handle_upload(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let dir = std::env::current_dir().unwrap().join("uploads");
    if let Err(e) = create_dir_all(&dir).await {
        eprintln!("failed to create upload dir: {e}");
    }

    let mut multipart = match Multipart::new(req.headers(), req.into_body()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("multipart error: {e}");
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("invalid multipart"))
                .unwrap());
        }
    };

    while let Ok(Some(mut field)) = multipart.next_field().await {
        if let Some(name) = field.file_name().map(|s| s.to_string()) {
            let path = dir.join(name);
            match File::create(&path).await {
                Ok(mut file) => {
                    let mut written: u64 = 0;
                    while let Ok(Some(chunk)) = field.chunk().await {
                        written += chunk.len() as u64;
                        if written > MAX_UPLOAD_SIZE {
                            return Ok(Response::builder()
                                .status(StatusCode::PAYLOAD_TOO_LARGE)
                                .body(Body::from("file too large"))
                                .unwrap());
                        }
                        if let Err(e) = file.write_all(&chunk).await {
                            eprintln!("write error: {e}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("file create error: {e}");
                }
            }
        }
    }

    Ok(Response::new(Body::from("ok")))
}

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/upload") => handle_upload(req).await,
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("not found"))
            .unwrap()),
    }
}

pub async fn start() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = ([0, 0, 0, 0], PORT).into();
    let make_service =
        make_service_fn(|_| async { Ok::<_, Infallible>(service_fn(router)) });
    let server = Server::bind(&addr).serve(make_service);
    println!("HTTP server listening on http://{}", addr);
    tokio::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("server error: {e}");
        }
    });
    Ok(())
}
