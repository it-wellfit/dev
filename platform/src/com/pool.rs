use windows::Win32::System::Com::*;

/// COM object pool for managing object lifecycle and reuse
pub struct ObjectPool {
    objects: Vec<IDispatch>,
    max_capacity: usize,
}

impl ObjectPool {
    /// Create a new object pool with specified capacity
    pub fn new(max_capacity: usize) -> Self {
        Self {
            objects: Vec::with_capacity(max_capacity),
            max_capacity,
        }
    }
    
    /// Get a COM object from the pool or create a new one
    pub fn get(&mut self, prog_id: &str) -> Result<IDispatch, windows::core::Error> {
        if let Some(obj) = self.objects.pop() {
            Ok(obj)
        } else {
            // Pool is empty, create a new object
            Self::create_object(prog_id)
        }
    }
    
    /// Return a COM object to the pool for reuse
    pub fn return_to_pool(&mut self, obj: IDispatch) {
        if self.objects.len() < self.max_capacity {
            self.objects.push(obj);
        }
        // If pool is at capacity, drop the object (COM handles cleanup)
    }
    
    /// Create a new COM object
    fn create_object(prog_id: &str) -> Result<IDispatch, windows::core::Error> {
        let prog_id_hstring = HSTRING::from(prog_id);
        let mut obj: Option<IDispatch> = None;
        
        unsafe {
            CoCreateInstance(&prog_id_hstring, None, CLSCTX_LOCAL_SERVER, &IDispatch::IID, &mut obj as *mut _ as *mut _)?.copy_to(&mut obj)?;
        }
        
        Ok(obj.ok_or_else(|| windows::core::Error::new(0x80004003, "Failed to create COM object"))?)
    }
    
    /// Clean up all objects in the pool
    pub fn cleanup(&mut self) {
        self.objects.clear();
    }
}


