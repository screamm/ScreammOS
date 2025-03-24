// src/queue.rs
// Simple queue implementation for ScreammOS

use core::sync::atomic::{AtomicUsize, Ordering};

/// A lockless queue for simple data types, designed to be used
/// in interrupt contexts where spinlocks might cause deadlocks.
/// Uses a ring buffer implementation.
pub struct ArrayQueue<T> {
    buffer: [T; 100],                  // Fixed-size buffer
    head: AtomicUsize,                // Index for dequeue operations
    tail: AtomicUsize,                // Index for enqueue operations
    capacity: usize,                  // Maximum queue capacity (= buffer.len())
}

impl<T: Copy + Default> ArrayQueue<T> {
    /// Create a new queue with the given capacity
    pub fn new(capacity: usize) -> Self {
        assert!(capacity <= 100, "Capacity exceeds max buffer size");
        
        Self {
            buffer: [T::default(); 100],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            capacity,
        }
    }
    
    /// Add an item to the queue. Returns Err if the queue is full.
    pub fn push(&mut self, item: T) -> Result<(), ()> {
        let current_tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (current_tail + 1) % self.capacity;
        
        // Check if queue is full
        if next_tail == self.head.load(Ordering::Relaxed) {
            return Err(());
        }
        
        // Add the item
        self.buffer[current_tail] = item;
        
        // Update tail pointer (ensure memory ordering)
        self.tail.store(next_tail, Ordering::Release);
        
        Ok(())
    }
    
    /// Remove and return an item from the queue. Returns None if the queue is empty.
    pub fn pop(&mut self) -> Option<T> {
        let current_head = self.head.load(Ordering::Relaxed);
        
        // Check if queue is empty
        if current_head == self.tail.load(Ordering::Relaxed) {
            return None;
        }
        
        // Get the item
        let item = self.buffer[current_head];
        
        // Update head pointer (ensure memory ordering)
        self.head.store((current_head + 1) % self.capacity, Ordering::Release);
        
        Some(item)
    }
    
    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed)
    }
    
    /// Get the number of items in the queue
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        
        if tail >= head {
            tail - head
        } else {
            self.capacity - head + tail
        }
    }
    
    /// Get the capacity of the queue
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Clear the queue
    pub fn clear(&mut self) {
        self.head.store(0, Ordering::Relaxed);
        self.tail.store(0, Ordering::Relaxed);
    }
} 