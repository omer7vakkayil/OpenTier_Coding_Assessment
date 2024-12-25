# Test Suite for Enhanced Multi-threaded Server

use std::sync::Arc;
use std::thread;
use embedded_recruitment_task::{
    message::{client_message, server_message, AddRequest, EchoMessage},
    server::Server,
};

mod client;

/// Sets up a server instance and runs it in a separate thread.
fn setup_server_thread(server: Arc<Server>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        server.run().expect("Server encountered an error");
    })
}

/// Creates a new server instance.
fn create_server() -> Arc<Server> {
    Arc::new(Server::new("localhost:8080").expect("Failed to start server"))
}

/// Tests single client echo message functionality.
#[test]
fn test_single_client_echo() {
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let mut echo_message = EchoMessage::default();
    echo_message.content = "Hello, World!".to_string();
    let message = client_message::Message::EchoMessage(echo_message.clone());

    assert!(client.send(message).is_ok(), "Failed to send message");

    let response = client.receive();
    assert!(response.is_ok(), "Failed to receive response");

    match response.unwrap().message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(echo.content, echo_message.content, "Echoed message mismatch");
        }
        _ => panic!("Expected EchoMessage"),
    }

    assert!(client.disconnect().is_ok(), "Failed to disconnect");

    server.stop();
    assert!(handle.join().is_ok(), "Server thread join failed");
}

/// Tests single client AddRequest functionality.
#[test]
fn test_single_client_add_request() {
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let mut add_request = AddRequest::default();
    add_request.a = 15;
    add_request.b = 25;
    let message = client_message::Message::AddRequest(add_request.clone());

    assert!(client.send(message).is_ok(), "Failed to send message");

    let response = client.receive();
    assert!(response.is_ok(), "Failed to receive response");

    match response.unwrap().message {
        Some(server_message::Message::AddResponse(add_response)) => {
            assert_eq!(add_response.result, add_request.a + add_request.b, "Add result mismatch");
        }
        _ => panic!("Expected AddResponse"),
    }

    assert!(client.disconnect().is_ok(), "Failed to disconnect");

    server.stop();
    assert!(handle.join().is_ok(), "Server thread join failed");
}

/// Tests multiple clients sending messages concurrently.
#[test]
fn test_multiple_clients() {
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    let mut clients = vec![];
    for _ in 0..10 {
        let mut client = client::Client::new("localhost", 8080, 1000);
        assert!(client.connect().is_ok(), "Failed to connect");
        clients.push(client);
    }

    for client in clients.iter_mut() {
        let mut echo_message = EchoMessage::default();
        echo_message.content = "Hello from client!".to_string();
        let message = client_message::Message::EchoMessage(echo_message.clone());

        assert!(client.send(message).is_ok(), "Failed to send");
        let response = client.receive();
        assert!(response.is_ok(), "Failed to receive");

        match response.unwrap().message {
            Some(server_message::Message::EchoMessage(echo)) => {
                assert_eq!(echo.content, echo_message.content, "Echo mismatch");
            }
            _ => panic!("Expected EchoMessage"),
        }
    }

    for client in clients.iter_mut() {
        assert!(client.disconnect().is_ok(), "Failed to disconnect");
    }

    server.stop();
    assert!(handle.join().is_ok(), "Server thread join failed");
}

/// Ensures shared state consistency across clients.
#[test]
fn test_shared_state_consistency() {
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    let mut clients = vec![];
    for _ in 0..5 {
        let mut client = client::Client::new("localhost", 8080, 1000);
        assert!(client.connect().is_ok(), "Failed to connect");
        clients.push(client);
    }

    let test_message = "Consistency Test".to_string();

    for _ in 0..10 {
        for client in clients.iter_mut() {
            let mut echo_message = EchoMessage::default();
            echo_message.content = test_message.clone();
            let message = client_message::Message::EchoMessage(echo_message.clone());

            assert!(client.send(message).is_ok(), "Failed to send");
            let response = client.receive();
            assert!(response.is_ok(), "Failed to receive");

            match response.unwrap().message {
                Some(server_message::Message::EchoMessage(echo)) => {
                    assert_eq!(echo.content, test_message, "State consistency error");
                }
                _ => panic!("Expected EchoMessage"),
            }
        }
    }

    for client in clients.iter_mut() {
        assert!(client.disconnect().is_ok(), "Failed to disconnect");
    }

    server.stop();
    assert!(handle.join().is_ok(), "Server thread join failed");
}
