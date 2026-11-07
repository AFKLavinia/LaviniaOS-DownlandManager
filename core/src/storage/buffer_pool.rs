use parking_lot::Mutex;
use std::sync::Arc;

/// A reusable pool of raw byte buffers to eliminate heap allocation overhead in high-throughput streaming.
#[derive(Clone)]
pub struct BufferPool {
    chunk_size: usize,
    max_buffers: usize,
    pool: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl BufferPool {
    /// Create a new pool with fixed chunk size (default 64KB) and maximum capacity.
    pub fn new(chunk_size: usize, max_buffers: usize) -> Self {
        Self {
            chunk_size,
            max_buffers,
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_buffers))),
        }
    }

    /// Acquire a buffer from the pool or allocate a new one if pool is empty.
    pub fn acquire(&self) -> PooledBuffer {
        let mut guard = self.pool.lock();
        let buffer = guard.pop().unwrap_or_else(|| vec![0u8; self.chunk_size]);
        PooledBuffer {
            buffer,
            pool: self.clone(),
        }
    }

    /// Return a buffer to the pool.
    fn release(&self, mut buf: Vec<u8>) {
        if buf.capacity() >= self.chunk_size {
            buf.resize(self.chunk_size, 0);
            let mut guard = self.pool.lock();
            if guard.len() < self.max_buffers {
                guard.push(buf);
            }
        }
    }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        // 64 KB buffers, pool up to 128 buffers (~8MB RAM max)
        Self::new(64 * 1024, 128)
    }
}

/// RAII wrapper that automatically returns the buffer to the pool on drop.
pub struct PooledBuffer {
    pub buffer: Vec<u8>,
    pool: BufferPool,
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        let buf = std::mem::take(&mut self.buffer);
        self.pool.release(buf);
    }
}

impl std::ops::Deref for PooledBuffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl std::ops::DerefMut for PooledBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}
