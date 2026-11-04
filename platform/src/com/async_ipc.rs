use tokio::net::UnixStream;
use std::time::Duration;

/// Async IPC communication for COM bridge
pub struct AsyncIpc {
    socket_path: String,
    timeout: Duration,
}

impl AsyncIpc {
    /// Create a new async IPC instance
    pub fn new(socket_path: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
            timeout: Duration::from_millis(5000), // 5 seconds timeout
        }
    }
    
    /// Connect to the IPC socket
    pub async fn connect(&self) -> Result<UnixStream, std::io::Error> {
        UnixStream::connect(&self.socket_path).await
    }
    
    /// Send a message and receive response asynchronously
    pub async fn send_receive(&self, message: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let mut stream = self.connect().await?;
        
        // Send message
        stream.write_all(message).await?;
        
        // Receive response with timeout
        let mut response = Vec::new();
        tokio::select! {
            result = stream.read_to_end(&mut response) => {
                result.map_err(std::io::Error::from)?;
            }
            _ = tokio::time::sleep(self.timeout) => {
                return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "IPC timeout"));
            }
        }
        
        Ok(response)
    }
}
