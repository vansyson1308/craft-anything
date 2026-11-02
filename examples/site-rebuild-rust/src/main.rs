//! Kinora: an AI creative-suite landing page, rebuilt in Rust with Craft Anything.
//!
//! `kinora-site [serve] [--addr 127.0.0.1:8080]` serves the site; `kinora-site export <folder>`
//! writes it as static files any web host can serve.

#![forbid(unsafe_code)]

mod art;
mod content;
mod pages;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use axum::Router;
use axum::extract::Path as UrlPath;
use axum::http::{HeaderValue, StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;

const STYLE: &str = include_str!("../assets/style.css");
const SCRIPT: &str = include_str!("../assets/app.js");

fn router() -> Router {
    Router::new()
        .route("/", get(|| async { Html(pages::home().into_string()) }))
        .route("/static/style.css", get(|| async { asset("text/css; charset=utf-8", STYLE) }))
        .route("/static/app.js", get(|| async { asset("text/javascript; charset=utf-8", SCRIPT) }))
        .route("/favicon.svg", get(|| async { asset("image/svg+xml", pages::favicon()) }))
        .route("/art/{name}", get(art_handler))
        .route("/healthz", get(|| async { "ok" }))
        .fallback(page_handler)
}

fn asset(content_type: &'static str, body: impl Into<String>) -> Response {
    let mut r = body.into().into_response();
    r.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    r.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=3600"));
    r
}

/// Parses `<seed>-<w>x<h>.svg`.
fn parse_art(name: &str) -> Option<(u32, u32, u32)> {
    let stem = name.strip_suffix(".svg")?;
    let (seed, size) = stem.split_once('-')?;
    let (w, h) = size.split_once('x')?;
    Some((seed.parse().ok()?, w.parse().ok()?, h.parse().ok()?))
}

async fn art_handler(UrlPath(name): UrlPath<String>) -> Response {
    match parse_art(&name) {
        Some((seed, w, h)) => asset("image/svg+xml", art::svg(seed, w, h)),
        None => (StatusCode::NOT_FOUND, Html(pages::not_found().into_string())).into_response(),
    }
}

async fn page_handler(uri: Uri) -> Response {
    let path = uri.path().trim_end_matches('/');
    match content::known_page(path) {
        Some(title) => Html(pages::placeholder(title).into_string()).into_response(),
        None => (StatusCode::NOT_FOUND, Html(pages::not_found().into_string())).into_response(),
    }
}

/// Writes the whole site as static files into `out`, which must not exist yet.
fn export(out: &Path) -> Result<usize, String> {
    if out.exists() {
        return Err(format!("{} already exists; pick a new folder", out.display()));
    }
    let write = |rel: &str, body: &str| -> Result<(), String> {
        let path = out.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("couldn't create {}: {e}", parent.display()))?;
        }
        std::fs::write(&path, body).map_err(|e| format!("couldn't write {}: {e}", path.display()))
    };
    let mut n = 0usize;
    write("index.html", &pages::home().into_string())?;
    write("404.html", &pages::not_found().into_string())?;
    write("static/style.css", STYLE)?;
    write("static/app.js", SCRIPT)?;
    write("favicon.svg", &pages::favicon())?;
    n += 5;
    for (seed, w, h) in pages::all_art() {
        write(&format!("art/{seed}-{w}x{h}.svg"), &art::svg(seed, w, h))?;
        n += 1;
    }
    for path in content::all_pages() {
        let title = content::known_page(&path).unwrap_or("Page");
        write(&format!("{}/index.html", path.trim_start_matches('/')), &pages::placeholder(title).into_string())?;
        n += 1;
    }
    Ok(n)
}

enum Cmd {
    Serve(SocketAddr),
    Export(PathBuf),
    Help,
}

fn parse_args(args: &[String]) -> Result<Cmd, String> {
    let mut addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "serve" => {}
            "export" => {
                let dir = it.next().ok_or("export needs a folder: kinora-site export <folder>")?;
                return Ok(Cmd::Export(PathBuf::from(dir)));
            }
            "--addr" => {
                let v = it.next().ok_or("--addr needs a value, like 127.0.0.1:8080")?;
                addr = v.parse().map_err(|_| format!("{v} isn't an address like 127.0.0.1:8080"))?;
            }
            "-h" | "--help" | "help" => return Ok(Cmd::Help),
            other => return Err(format!("unknown argument {other}; try --help")),
        }
    }
    Ok(Cmd::Serve(addr))
}

const HELP: &str = "kinora-site: an AI creative-suite landing page, rebuilt in Rust

USAGE:
  kinora-site [serve] [--addr 127.0.0.1:8080]   serve the site
  kinora-site export <folder>                   write it as static files";

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_args(&args) {
        Ok(Cmd::Help) => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        Ok(Cmd::Export(dir)) => match export(&dir) {
            Ok(n) => {
                println!("Wrote {n} files to {}", dir.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        Ok(Cmd::Serve(addr)) => {
            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("error: couldn't listen on {addr}: {e}. Is something else using that port? Try --addr 127.0.0.1:8081");
                    return ExitCode::FAILURE;
                }
            };
            println!("Serving on http://{addr} (Ctrl+C to stop)");
            let shutdown = async {
                let _ = tokio::signal::ctrl_c().await;
            };
            match axum::serve(listener, router()).with_graceful_shutdown(shutdown).await {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("error: the server stopped: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn get_path(path: &str) -> (StatusCode, String, String) {
        let req = match Request::builder().uri(path).body(Body::empty()) {
            Ok(r) => r,
            Err(e) => return (StatusCode::BAD_REQUEST, String::new(), e.to_string()),
        };
        let res = match router().oneshot(req).await {
            Ok(r) => r,
            Err(e) => match e {},
        };
        let status = res.status();
        let ct = res.headers().get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
        let body = res.into_body().collect().await.map(|b| b.to_bytes()).unwrap_or_default();
        (status, ct, String::from_utf8_lossy(&body).into_owned())
    }

    #[tokio::test]
    async fn home_page_has_every_section() {
        let (status, ct, body) = get_path("/").await;
        assert_eq!(status, StatusCode::OK);
        assert!(ct.starts_with("text/html"));
        for needle in [
            "<main id=\"main\"",
            "Weave by the community",
            "See inside every project",
            "One board.",
            "More ways to create",
            "The AI creative",
            "<header",
            "<footer",
        ] {
            assert!(body.contains(needle), "missing {needle}");
        }
    }

    #[tokio::test]
    async fn every_picture_and_link_on_the_home_page_resolves() {
        let (_, _, body) = get_path("/").await;
        let mut checked = 0;
        for attr in ["src=\"", "href=\""] {
            for part in body.split(attr).skip(1) {
                let url = part.split('"').next().unwrap_or("");
                if !url.starts_with('/') {
                    continue;
                }
                let (status, _, _) = get_path(url).await;
                assert_eq!(status, StatusCode::OK, "{url}");
                checked += 1;
            }
        }
        assert!(checked > 100, "only {checked} links");
    }

    #[tokio::test]
    async fn bad_addresses_are_404_not_crashes() {
        for p in ["/nope", "/art/x.svg", "/art/1-99999999999x1.svg", "/art/-1x1.svg", "/art/1-1x.svg"] {
            let (status, _, _) = get_path(p).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{p}");
        }
        let (status, ct, _) = get_path("/art/5-0x0.svg").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ct, "image/svg+xml");
    }

    #[test]
    fn export_writes_a_static_site() {
        let Ok(tmp) = std::env::temp_dir().canonicalize() else { return };
        let out = tmp.join(format!("kinora-export-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&out);
        let n = export(&out);
        assert!(matches!(n, Ok(n) if n > 100), "{n:?}");
        assert!(out.join("index.html").is_file());
        assert!(out.join("pricing/index.html").is_file());
        assert!(export(&out).is_err(), "an existing folder is refused");
        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn arguments() {
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert!(matches!(parse_args(&s(&[])), Ok(Cmd::Serve(_))));
        assert!(matches!(parse_args(&s(&["--addr", "0.0.0.0:9000"])), Ok(Cmd::Serve(a)) if a.port() == 9000));
        assert!(parse_args(&s(&["--addr"])).is_err());
        assert!(parse_args(&s(&["--addr", "nope"])).is_err());
        assert!(parse_args(&s(&["export"])).is_err());
        assert!(matches!(parse_args(&s(&["export", "dist"])), Ok(Cmd::Export(_))));
    }
}
