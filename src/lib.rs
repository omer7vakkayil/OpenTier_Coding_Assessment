pub mod message;
pub mod server;

use log::{error, info};
use prost::Message;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    thread,
    time::Duration,
};

/// Core Server Logic
pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
}

impl Server {
    /// Create a new server instance
    pub fn new(addr: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;

        Ok(Server {
            listener,
            is_running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Run the server to accept client connections
    pub fn run(&self) -> std::io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst);
        info!("Server is running on {}", self.listener.local_addr()?);

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);
                    let is_running = self.is_running.clone();
                    thread::spawn(move || {
                        if let Err(e) = handle_client(stream, is_running) {
                            error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        info!("Server stopped.");
        Ok(())
    }

    /// Stop the server
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        info!("Stopping the server...");
    }
}

/// Handle individual client communication
fn handle_client(mut stream: TcpStream, is_running: Arc<AtomicBool>) -> std::io::Result<()> {
    while is_running.load(Ordering::SeqCst) {
        let mut buffer = [0u8; 1024];
        let bytes_read = stream.read(&mut buffer)?;

        if bytes_read == 0 {
            info!("Client disconnected: {}", stream.peer_addr()?);
            break;
        }

        info!(
            "Received {} bytes from client {}",
            bytes_read,
            stream.peer_addr()?
        );

        // Decode the message
        let request = match message::client_message::Message::decode(&buffer[..bytes_read]) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to decode message: {}", e);
                continue;
            }
        };

        // Handle the request and form a response
        let response = match handle_message(request) {
            Ok(response) => response,
            Err(e) => {
                error!("Error handling message: {}", e);
                continue;
            }
        };

        // Encode and send the response
        let mut response_buffer = Vec::new();
        response.encode(&mut response_buffer)?;
        stream.write_all(&response_buffer)?;
    }

    Ok(())
}

/// Process incoming messages and prepare responses
fn handle_message(request: message::client_message::Message) -> Result<message::server_message::Message, String> {
    match request {
        message::client_message::Message::EchoMessage(echo_msg) => {
            let mut response = message::server_message::EchoMessage::default();
            response.content = echo_msg.content;
            Ok(message::server_message::Message::EchoMessage(response))
        }
        message::client_message::Message::AddRequest(add_req) => {
            let mut response = message::server_message::AddResponse::default();
            response.result = add_req.a + add_req.b;
            Ok(message::server_message::Message::AddResponse(response))
        }
        _ => Err("Unsupported message type".to_string()),
    }
}
