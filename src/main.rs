use embedded_recruitment_task::{
    message::{client_message, server_message, EchoMessage, ServerMessage},
    server::Client,
};
use log::{error, info};
use std::io::{self, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread;
use std::time::Duration;

// Server Structure
pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
}

impl Server {
    // Create a new server instance
    pub fn new(address: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        Ok(Server {
            listener,
            is_running: Arc::new(AtomicBool::new(false)),
        })
    }

    // Start the server
    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst);
        info!("Server is running on {}", self.listener.local_addr()?);

        let is_running = self.is_running.clone();

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    info!("New client connected: {}", stream.peer_addr()?);
                    let is_running = is_running.clone();

                    // Spawn a thread to handle each client
                    thread::spawn(move || {
                        if let Err(e) = handle_client(stream, is_running) {
                            error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100)); // Allow other operations
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }

            if !is_running.load(Ordering::SeqCst) {
                break;
            }
        }

        info!("Server stopped.");
        Ok(())
    }

    // Stop the server
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        info!("Stopping server...");
    }
}

// Handle individual client connections
fn handle_client(mut stream: TcpStream, is_running: Arc<AtomicBool>) -> io::Result<()> {
    let peer_addr = stream.peer_addr()?;
    info!("Handling client: {}", peer_addr);

    while is_running.load(Ordering::SeqCst) {
        let mut buffer = vec![0u8; 1024];

        match stream.read(&mut buffer) {
            Ok(0) => {
                info!("Client disconnected: {}", peer_addr);
                break;
            }
            Ok(size) => {
                info!("Received {} bytes from client: {}", size, peer_addr);

                // Decode the client message
                let message = client_message::Message::decode(&buffer[..size])
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                // Process the message and create a response
                let response = process_message(message);

                // Encode the response and send it back to the client
                let mut response_buffer = Vec::new();
                response.encode(&mut response_buffer)?;

                stream.write_all(&response_buffer)?;
                stream.flush()?;
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50)); // Prevent busy waiting
            }
            Err(e) => {
                error!("Error reading from client {}: {}", peer_addr, e);
                break;
            }
        }
    }

    info!("Finished handling client: {}", peer_addr);
    Ok(())
}

// Process a client message and create a server response
fn process_message(message: client_message::Message) -> ServerMessage {
    match message {
        client_message::Message::EchoMessage(echo) => {
            ServerMessage {
                message: Some(server_message::Message::EchoMessage(EchoMessage {
                    content: echo.content,
                })),
            }
        }
        client_message::Message::AddRequest(add) => {
            ServerMessage {
                message: Some(server_message::Message::AddResponse(
                    server_message::AddResponse {
                        result: add.a + add.b,
                    },
                )),
            }
        }
        _ => ServerMessage {
            message: None, // For unsupported message types
        },
    }
}
