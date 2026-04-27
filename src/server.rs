use axum::{routing::get_service, Router};
use std::net::SocketAddr;
use axum::http::{header, HeaderValue};
use tower_http::services::ServeFile;
use tower_http::set_header::SetResponseHeaderLayer;
use axum_server::tls_rustls::RustlsConfig;

pub async fn start(ip: std::net::IpAddr, port: u16, file_path: String) {
    let addr = SocketAddr::new(ip, port);

    // 1. Generate a self-signed certificate on the fly for your specific local IP
    let subject_alt_names = vec![ip.to_string()];
    let cert = rcgen::generate_simple_self_signed(subject_alt_names)
        .expect("Failed to generate TLS certificate");
    
    // Extract the raw bytes of the certificate and private key
    let cert_der = cert.serialize_der().unwrap();
    let priv_key_der = cert.serialize_private_key_der();

    // 2. Configure the server with our new temporary encryption keys
    let config = RustlsConfig::from_der(vec![cert_der], priv_key_der)
        .await
        .expect("Failed to build TLS config");

    // 3. Set up the file service (same as before)
    let service = get_service(ServeFile::new(&file_path))
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_static("attachment"),
        ));

    let app = Router::new().route("/download", service);

    // 4. Bind using HTTPS instead of HTTP
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}