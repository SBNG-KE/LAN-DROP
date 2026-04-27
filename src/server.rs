use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use axum_server::tls_rustls::RustlsConfig;
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};
use tokio::{fs::File, sync::RwLock};
use tokio_util::io::ReaderStream;

pub type SharedFile = Arc<RwLock<PathBuf>>;

pub async fn start(ip: IpAddr, port: u16, shared_file: SharedFile) {
    let addr = SocketAddr::new(ip, port);

    let subject_alt_names = vec![ip.to_string()];
    let cert = rcgen::generate_simple_self_signed(subject_alt_names)
        .expect("Failed to generate TLS certificate");

    let cert_der = cert.serialize_der().unwrap();
    let priv_key_der = cert.serialize_private_key_der();

    let config = RustlsConfig::from_der(vec![cert_der], priv_key_der)
        .await
        .expect("Failed to build TLS config");

    let app = Router::new()
        .route("/download", get(download_current_file))
        .with_state(shared_file);

    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn download_current_file(State(shared_file): State<SharedFile>) -> Response {
    let file_path = shared_file.read().await.clone();
    let file = match File::open(&file_path).await {
        Ok(file) => file,
        Err(_) => {
            return (StatusCode::NOT_FOUND, "Selected file could not be opened.").into_response();
        }
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let file_name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download");
    let disposition = format!("attachment; filename=\"{}\"", file_name.replace('"', ""));

    let mut response = body.into_response();
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition)
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    response
}
