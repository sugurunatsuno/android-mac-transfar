use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use multer::{Field, Multipart};
use tokio::fs::{create_dir_all, metadata, File};
use tokio::io::AsyncWriteExt;
use std::path::{Path, PathBuf};
use tokio::sync::{broadcast, RwLock};
use once_cell::sync::Lazy;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use serde::Deserialize;
use serde_json::json;
use local_ip_address::list_afinet_netifas;

// Static client assets served at '/'
static INDEX_HTML: &str = include_str!("../../client/index.html");
static MAIN_JS: &str = include_str!("../../client/main.js");

/// Port number the HTTP server listens on.
pub const PORT: u16 = 8080;

/// Maximum allowed upload size in bytes (4 GiB).
pub const MAX_UPLOAD_SIZE: u64 = 4 * 1024 * 1024 * 1024;

static UPLOAD_DIR: Lazy<RwLock<PathBuf>> = Lazy::new(|| {
    let path = std::env::current_dir().unwrap().join("uploads");
    RwLock::new(path)
});

static EVENT_TX: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
    let (tx, _rx) = broadcast::channel(100);
    tx
});

fn notify(event: &str) {
    let _ = EVENT_TX.send(event.to_string());
}

fn subscribe_events() -> broadcast::Receiver<String> {
    EVENT_TX.subscribe()
}

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

    notify(&format!("{{\"file\":\"{}\",\"status\":\"start\"}}", name));

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
        notify(&format!("{{\"file\":\"{}\",\"status\":\"progress\",\"bytes\":{}}}", name, written));
    }
    notify(&format!("{{\"file\":\"{}\",\"status\":\"done\"}}", name));
    Ok(path)
}

async fn handle_upload(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let dir = UPLOAD_DIR.read().await.clone();
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

async fn handle_events(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let stream = BroadcastStream::new(subscribe_events())
        .filter_map(|res| async move { res.ok() })
        .map(|msg| Ok::<_, Infallible>(hyper::body::Bytes::from(format!("data: {}\n\n", msg))));
    let body = Body::wrap_stream(stream);
    Ok(Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("connection", "keep-alive")
        .body(body)
        .unwrap())
}

async fn handle_info() -> Result<Response<Body>, Infallible> {
    let dir = UPLOAD_DIR.read().await.clone();
    let ips = list_afinet_netifas()
        .map(|m| m.into_iter().map(|(_, ip)| ip.to_string()).collect::<Vec<_>>())
        .unwrap_or_else(|_| Vec::new());
    let body = json!({
        "ips": ips,
        "port": PORT,
        "dir": dir,
    });
    Ok(Response::builder()
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap())
}

#[derive(Deserialize)]
struct DirPayload { dir: String }

async fn handle_set_dir(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
    if let Ok(payload) = serde_json::from_slice::<DirPayload>(&bytes) {
        let mut dir = UPLOAD_DIR.write().await;
        *dir = PathBuf::from(payload.dir);
        let _ = create_dir_all(&*dir).await;
        Ok(Response::new(Body::from("ok")))
    } else {
        Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("invalid"))
            .unwrap())
    }
}

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::builder()
            .header("content-type", "text/html")
            .body(Body::from(INDEX_HTML))
            .unwrap()),
        (&Method::GET, "/main.js") => Ok(Response::builder()
            .header("content-type", "application/javascript")
            .body(Body::from(MAIN_JS))
            .unwrap()),
        (&Method::POST, "/upload") => handle_upload(req).await,
        (&Method::GET, "/events") => handle_events(req).await,
        (&Method::GET, "/info") => handle_info().await,
        (&Method::POST, "/set_dir") => handle_set_dir(req).await,
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
