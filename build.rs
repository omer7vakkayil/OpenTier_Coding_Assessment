fn main() {
    // Compile Protocol Buffers (.proto) files
    tonic_build::configure()
        .build_server(true)
        .compile(&["proto/messages.proto"], &["proto/"])
        .unwrap_or_else(|e| panic!("Failed to compile .proto files: {}", e));
}
