use qr2term::print_qr;

pub fn print_url_qr(url: &str) {
    // qr2term handles the terminal formatting automatically
    if print_qr(url).is_err() {
        eprintln!("Failed to generate QR code in terminal, but server is still running.");
    }
}
