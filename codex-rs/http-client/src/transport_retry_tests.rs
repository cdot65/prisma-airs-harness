//! Slow diagnostic bodies must consume, not restart, server retry advice.

use super::*;
use pretty_assertions::assert_eq;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
enum Delivery {
    Buffered,
    Streamed,
}

#[tokio::test]
async fn deadline_is_captured_before_reading_the_error_body() {
    for delivery in [Delivery::Buffered, Delivery::Streamed] {
        for body_limit in [None, Some(64)] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let url = format!("http://{}/responses", listener.local_addr().unwrap());
            let server = std::thread::spawn(move || {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    let mut byte = [0];
                    socket.read_exact(&mut byte).unwrap();
                    request.push(byte[0]);
                }
                socket.write_all(b"HTTP/1.1 503 Service Unavailable\r\nRetry-After: 1\r\nContent-Length: 4\r\nConnection: close\r\n\r\n").unwrap();
                socket.flush().unwrap();
                // Leave a full second of scheduling margin after the advised deadline.
                std::thread::sleep(Duration::from_secs(2));
                socket.write_all(b"busy").unwrap();
            });
            let transport =
                ReqwestTransport::new(reqwest::Client::builder().no_proxy().build().unwrap());
            let mut request = Request::new(Method::GET, url);
            request.timeout = Some(Duration::from_secs(10));
            request.response_body_limit_bytes = body_limit;
            let error = match delivery {
                Delivery::Buffered => transport.execute(request).await.err().unwrap(),
                Delivery::Streamed => transport.stream(request).await.err().unwrap(),
            };
            server.join().unwrap();
            let TransportError::Http {
                status,
                body,
                retry_after,
                ..
            } = error
            else {
                panic!("expected HTTP error");
            };
            assert_eq!(
                (status, body),
                (StatusCode::SERVICE_UNAVAILABLE, Some("busy".into()))
            );
            assert_eq!(
                retry_after.unwrap().remaining_delay(),
                Duration::ZERO,
                "{delivery:?}, limit={body_limit:?}"
            );
        }
    }
}
