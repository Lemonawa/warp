use futures::io::{BufReader, Cursor};
use uuid::Uuid;

use super::{receive_message, send_message, ConnectionAddress, Request, Response};
use crate::service::ServiceId;

// --- ConnectionAddress ---

#[test]
fn connection_address_new_contains_warp_prefix() {
    let addr = ConnectionAddress::new();
    assert!(addr.0.starts_with("/tmp/warp-ipc-"));
    assert!(addr.0.ends_with(".sock"));
}

#[test]
fn connection_address_display() {
    let addr = ConnectionAddress::from("/tmp/test.sock".to_string());
    assert_eq!(format!("{addr}"), "/tmp/test.sock");
}

#[test]
fn connection_address_from_string() {
    let addr = ConnectionAddress::from("/tmp/my.sock".to_string());
    assert_eq!(addr.0, "/tmp/my.sock");
}

// --- Request ---

#[test]
fn request_new_generates_unique_ids() {
    let r1 = Request::new("svc1".to_string(), vec![1, 2, 3]);
    let r2 = Request::new("svc1".to_string(), vec![1, 2, 3]);
    assert_ne!(r1.id(), r2.id());
}

#[test]
fn request_preserves_service_id_and_bytes() {
    let svc: ServiceId = "my_service".to_string();
    let data = vec![10, 20, 30];
    let req = Request::new(svc.clone(), data.clone());
    assert_eq!(req.service_id, svc);
    assert_eq!(req.bytes, data);
}

// --- Response ---

#[test]
fn response_success_has_correct_fields() {
    let req_id = Uuid::new_v4();
    let svc: ServiceId = "my_service".to_string();
    let resp = Response::success(req_id, svc.clone(), vec![42]);
    match resp {
        Response::Success {
            request_id,
            service_id,
            bytes,
        } => {
            assert_eq!(request_id, req_id);
            assert_eq!(service_id, svc);
            assert_eq!(bytes, vec![42]);
        }
        _ => panic!("Expected Success variant"),
    }
}

#[test]
fn response_failure_has_correct_fields() {
    let req_id = Uuid::new_v4();
    let resp = Response::failure(req_id, "something went wrong".to_string());
    match resp {
        Response::Failure {
            request_id,
            error_message,
        } => {
            assert_eq!(request_id, req_id);
            assert_eq!(error_message, "something went wrong");
        }
        _ => panic!("Expected Failure variant"),
    }
}

// --- send_message / receive_message roundtrip ---

#[tokio::test]
async fn send_and_receive_string_message_roundtrip() {
    let message = "hello world".to_string();

    let mut buf: Vec<u8> = Vec::new();
    send_message(&mut buf, message.clone()).await.unwrap();

    let mut reader = BufReader::new(Cursor::new(buf));
    let received: String = receive_message(&mut reader).await.unwrap();
    assert_eq!(received, message);
}

#[tokio::test]
async fn send_and_receive_vec_message_roundtrip() {
    let message: Vec<u8> = vec![1, 2, 3, 4, 5];

    let mut buf: Vec<u8> = Vec::new();
    send_message(&mut buf, message.clone()).await.unwrap();

    let mut reader = BufReader::new(Cursor::new(buf));
    let received: Vec<u8> = receive_message(&mut reader).await.unwrap();
    assert_eq!(received, message);
}

#[tokio::test]
async fn send_and_receive_request_roundtrip() {
    let req = Request::new("test_svc".to_string(), vec![10, 20]);

    let mut buf: Vec<u8> = Vec::new();
    send_message(&mut buf, req.clone()).await.unwrap();

    let mut reader = BufReader::new(Cursor::new(buf));
    let received: Request = receive_message(&mut reader).await.unwrap();
    assert_eq!(received.id, req.id);
    assert_eq!(received.service_id, req.service_id);
    assert_eq!(received.bytes, req.bytes);
}

#[tokio::test]
async fn send_and_receive_response_roundtrip() {
    let req_id = Uuid::new_v4();
    let resp = Response::success(req_id, "svc".to_string(), vec![99]);

    let mut buf: Vec<u8> = Vec::new();
    send_message(&mut buf, resp.clone()).await.unwrap();

    let mut reader = BufReader::new(Cursor::new(buf));
    let received: Response = receive_message(&mut reader).await.unwrap();
    match received {
        Response::Success {
            request_id,
            service_id,
            bytes,
        } => {
            assert_eq!(request_id, req_id);
            assert_eq!(service_id, "svc");
            assert_eq!(bytes, vec![99]);
        }
        _ => panic!("Expected Success variant"),
    }
}

#[tokio::test]
async fn receive_from_truncated_buffer_errors() {
    // Write only a partial message (just the header, no payload)
    let message = "test".to_string();
    let serialized = bincode::serialize(&message).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&serialized.len().to_be_bytes());
    // Intentionally omit the payload

    let mut reader = BufReader::new(Cursor::new(buf));
    let result: Result<String, _> = receive_message(&mut reader).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn send_and_receive_empty_string() {
    let message = String::new();

    let mut buf: Vec<u8> = Vec::new();
    send_message(&mut buf, message.clone()).await.unwrap();

    let mut reader = BufReader::new(Cursor::new(buf));
    let received: String = receive_message(&mut reader).await.unwrap();
    assert_eq!(received, message);
}
