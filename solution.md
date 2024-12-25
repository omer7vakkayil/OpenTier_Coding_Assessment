# **Approach**
1. Implemented a multithreaded server using Rust's `tokio` and `tonic` libraries.
2. Used Protocol Buffers for message serialization and deserialization.
3. Added test cases for concurrent client connections and edge cases.

## **Challenges**
1. Handling synchronization between threads while ensuring thread safety.
2. Debugging asynchronous code due to its complexity.
3. Ensuring the `.proto` file compiles correctly using `prost-build`.

## **Solutions**
- Used `tokio::sync::Mutex` and `Arc` to manage shared state.
- Followed Rust's concurrency guidelines to avoid deadlocks.
- Verified the `.proto` file with `protoc` before integrating it into the build system.

## **Results**
- The server passed all provided and additional test cases.
- Demonstrated support for handling concurrent client connections efficiently.
- Maintained clean and well-documented code for maintainability.

## **Future Improvements**
- Enhance logging for better debugging.
- Add support for more complex Protocol Buffer messages.
- Optimize the server for higher throughput and lower latency.

## **Test Suite Results**
- **Total Test Cases**: 6
- **Passed**: 6
- **Failed**: 0

### **Commands to Reproduce Results**
1. Run the server:
   ```bash
   cargo run