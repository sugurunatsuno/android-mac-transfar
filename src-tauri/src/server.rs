use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use multer::{Field, Multipart};
use tokio::fs::{create_dir_all, metadata, File};
use tokio::io::AsyncWriteExt;
use std::path::{Path, PathBuf};

/// Port number the HTTP server listens on.
pub const PORT: u16 = 8080;

/// Maximum allowed upload size in bytes (4 GiB).
pub const MAX_UPLOAD_SIZE: u64 = 4 * 1024 * 1024 * 1024;

/// Returns the port the server listens on.
pub fn port() -> u16 {
    PORT
}

/// Saves a multipart file field to `dir`, returning the path used.
/// If a file with the same name already exists, a numeric suffix is appended
/// before the extension.
async fn save_field(mut field: Field, dir: &Path) -> std::io::Result<PathBuf> {
    let name = field
        .file_name()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "file".into());

    let mut path = dir.join(&name);
    if metadata(&path).await.is_ok() {
        let stem = Path::new(&name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let ext = Path::new(&name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let mut i = 1;
        loop {
            let candidate = if ext.is_empty() {
                dir.join(format!("{}_{}", stem, i))
            } else {
                dir.join(format!("{}_{i}.{}", stem, ext))
            };
            if metadata(&candidate).await.is_err() {
                path = candidate;
                break;
            }
            i += 1;
        }
    }

    let mut file = File::create(&path).await?;
    let mut written: u64 = 0;
    while let Some(chunk) = field.chunk().await? {
        written += chunk.len() as u64;
        if written > MAX_UPLOAD_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "file too large",
            ));
        }
        file.write_all(&chunk).await?;
    }
    Ok(path)
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

    while let Ok(Some(field)) = multipart.next_field().await {
        match save_field(field, &dir).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::Other && e.to_string() == "file too large" => {
                return Ok(Response::builder()
                    .status(StatusCode::PAYLOAD_TOO_LARGE)
                    .body(Body::from("file too large"))
                    .unwrap());
            }
            Err(e) => {
                eprintln!("file save error: {e}");
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
