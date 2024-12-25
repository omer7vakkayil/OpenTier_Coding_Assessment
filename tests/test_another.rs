// Edge Case: Large Message Handling
#[test]
fn test_large_message() {
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect().is_ok());

    let large_message = "a".repeat(10_000); // 10 KB message
    let mut echo_message = EchoMessage::default();
    echo_message.content = large_message.clone();
    let message = client_message::Message::EchoMessage(echo_message);

    assert!(client.send(message).is_ok());
    let response = client.receive();
    assert!(response.is_ok());

    match response.unwrap().message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(echo.content, large_message);
        }
        _ => panic!("Expected EchoMessage, but received something else"),
    }

    client.disconnect().unwrap();
    server.stop();
    handle.join().unwrap();
}
