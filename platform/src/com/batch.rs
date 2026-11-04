use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Batch execution queue for COM operations
pub struct BatchExecutor {
    queue: VecDeque<ComRequest>,
    max_batch_size: usize,
    timeout: Duration,
    workers: usize,
}

impl BatchExecutor {
    /// Create a new batch executor
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            max_batch_size: 10,
            timeout: Duration::from_millis(1000), // 1 second timeout per batch
            workers: 4, // Number of worker threads
        }
    }
    
    /// Add a request to the batch queue
    pub fn add_request(&mut self, request: ComRequest) {
        if self.queue.len() < self.max_batch_size {
            self.queue.push_back(request);
        } else {
            // Queue is full, could implement priority or drop
        }
    }
    
    /// Execute all pending requests in batches
    pub async fn execute_batch(&mut self) -> Result<Vec<Variant>, windows::core::Error> {
        if self.queue.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut results = Vec::new();
        let batch: Vec<ComRequest> = self.queue.drain(..).collect();
        
        // Process batch in parallel
        let mut handles = Vec::new();
        
        for request in batch {
            let handle = tokio::spawn(Self::process_single_request(request));
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(windows::core::Error::new(0x80004005, format!("Batch execution failed: {:?}", e))),
            }
        }
        
        Ok(results)
    }
    
    /// Process a single COM request
    async fn process_single_request(request: ComRequest) -> Result<Variant, windows::core::Error> {
        // Create COM object
        let mut obj = ComBridge::create_object(&request.prog_id)?;
        
        // Execute method with timeout
        let start_time = Instant::now();
        
        let result = tokio::select! {
            result = Self::invoke_method(&mut obj, &request.method_name, &request.args) => {
                result
            }
            _ = tokio::time::sleep(request.timeout_ms.unwrap_or(Self::timeout)) => {
                // Timeout occurred
                Self::cleanup_object(&mut obj);
                return Err(windows::core::Error::new(0x80070102, "Method invocation timeout"));
            }
        };
        
        // Check if we exceeded timeout
        if start_time.elapsed() > Self::timeout {
            Self::cleanup_object(&mut obj);
            return Err(windows::core::Error::new(0x80070102, "Batch timeout exceeded"));
        }
        
        Ok(result)
    }
    
    /// Invoke a method on a COM object
    async fn invoke_method(
        obj: &mut IDispatch,
        method_name: &str,
        args: &[Variant]
    ) -> Result<Variant, windows::core::Error> {
        // Convert Rust variants to Windows VARIANTs
        let mut native_args = Vec::new();
        
        for arg in args {
            native_args.push(arg.to_variant());
        }
        
        // Invoke the method
        let mut result_var = Variant::default();
        
        unsafe {
            obj.Invoke(method_name, &native_args, &mut result_var)?;
        }
        
        Ok(result_var)
    }
    
    /// Clean up a COM object
    fn cleanup_object(obj: &mut IDispatch) {
        // Release the COM object
        let _ = obj;
    }
}

