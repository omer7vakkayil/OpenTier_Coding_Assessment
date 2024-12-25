use log::{error, info, warn};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512];
        // Read data from the client
        let bytes_read = self.stream.read(&mut buffer)?;
        if bytes_read == 0 {
            info!("Client disconnected.");
            return Ok(());
        }

        if let Ok(message) = EchoMessage::decode(&buffer[..bytes_read]) {
            info!("Received: {}", message.content);
            // Echo back the message
            let payload = message.encode_to_vec();
            self.stream.write_all(&payload)?;
            self.stream.flush()?;
        } else {
            error!("Failed to decode message");
        }

        Ok(())
    }
}

pub struct Server {
    listener: TcpListener,
    is_running: Arc<Mutex<bool>>,
}

impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let is_running = Arc::new(Mutex::new(true));
        Ok(Server { listener, is_running })
    }

    /// Runs the server, listening for incoming connections and handling them
    pub fn run(&self) -> io::Result<()> {
        info!("Server is running on {}", self.listener.local_addr()?);

        let is_running = Arc::clone(&self.is_running);
        for stream in self.listener.incoming() {
            let is_running = Arc::clone(&is_running);

            match stream {
                Ok(stream) => {
                    info!("New client connected: {}", stream.peer_addr()?);

                    // Spawn a new thread to handle the client
                    thread::spawn(move || {
                        let mut client = Client::new(stream);

                        loop {
                            if !*is_running.lock().unwrap() {
                                break;
                            }

                            if let Err(e) = client.handle() {
                                error!("Error handling client: {}", e);
                                break;
                            }
                        }

                        info!("Client thread exiting.");
                    });
                }
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        // Non-blocking accept, sleep briefly to reduce CPU usage
                        thread::sleep(Duration::from_millis(100));
                    } else {
                        error!("Error accepting connection: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Stops the server by setting the `is_running` flag to `false`
    pub fn stop(&self) {
        let mut running = self.is_running.lock().unwrap();
        if *running {
            *running = false;
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}

/// EchoMessage definition (replace with your protobuf definition)
#[derive(Clone, PartialEq, Message)]
pub struct EchoMessage {
    #[prost(string, tag = "1")]
    pub content: String,
}
