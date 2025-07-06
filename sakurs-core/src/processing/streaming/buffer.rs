//! Buffered reading with look-ahead for sentence boundary detection

use std::io::{self, Read};

/// Buffer for streaming text processing with look-ahead capability
pub struct StreamingBuffer {
    /// Main data buffer
    data: Vec<u8>,
    /// Buffer capacity
    capacity: usize,
    /// Current position in buffer
    position: usize,
    /// Look-ahead buffer for boundary detection
    look_ahead: Vec<u8>,
    /// Size of look-ahead buffer
    look_ahead_size: usize,
}

impl StreamingBuffer {
    /// Create a new streaming buffer
    pub fn new(buffer_size: usize, look_ahead_size: usize) -> Self {
        Self {
            data: Vec::with_capacity(buffer_size),
            capacity: buffer_size,
            position: 0,
            look_ahead: Vec::with_capacity(look_ahead_size),
            look_ahead_size,
        }
    }

    /// Fill buffer from reader, preserving look-ahead data
    pub fn fill(&mut self, reader: &mut impl Read) -> io::Result<usize> {
        // Clear main buffer but preserve look-ahead
        self.data.clear();
        self.position = 0;

        // Copy look-ahead to start of buffer if exists
        if !self.look_ahead.is_empty() {
            self.data.extend_from_slice(&self.look_ahead);
        }

        // Calculate how much new data to read
        let space_available = self.capacity - self.data.len();
        let mut temp_buffer = vec![0u8; space_available];

        // Read new data
        let bytes_read = reader.read(&mut temp_buffer)?;
        if bytes_read > 0 {
            self.data.extend_from_slice(&temp_buffer[..bytes_read]);
        }

        // Update look-ahead for next iteration
        self.update_look_ahead();

        Ok(self.data.len())
    }

    /// Get the current processable chunk (excluding look-ahead portion)
    pub fn processable_chunk(&self) -> &[u8] {
        let end = self.data.len().saturating_sub(self.look_ahead_size);
        &self.data[self.position..end]
    }

    /// Get the full buffer including look-ahead
    pub fn full_buffer(&self) -> &[u8] {
        &self.data[self.position..]
    }

    /// Update look-ahead buffer with end of current data
    fn update_look_ahead(&mut self) {
        self.look_ahead.clear();

        if self.data.len() > self.look_ahead_size {
            let start = self.data.len() - self.look_ahead_size;
            self.look_ahead.extend_from_slice(&self.data[start..]);
        } else {
            // If buffer is smaller than look-ahead size, save entire buffer
            self.look_ahead.extend_from_slice(&self.data);
        }
    }

    /// Check if buffer has processable data
    pub fn has_data(&self) -> bool {
        self.position < self.data.len()
    }

    /// Get current buffer size
    pub fn len(&self) -> usize {
        self.data.len() - self.position
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reset buffer state
    pub fn reset(&mut self) {
        self.data.clear();
        self.position = 0;
        self.look_ahead.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_buffer_fill() {
        let mut buffer = StreamingBuffer::new(1024, 64);
        let data = b"Hello world. This is a test sentence.";
        let mut reader = Cursor::new(data);

        let bytes_read = buffer.fill(&mut reader).unwrap();
        assert_eq!(bytes_read, data.len());
        assert_eq!(buffer.full_buffer(), data);
    }

    #[test]
    fn test_look_ahead_preservation() {
        let mut buffer = StreamingBuffer::new(100, 10);

        // First fill
        let data1 = b"First chunk of data with some text.";
        let mut reader1 = Cursor::new(data1);
        buffer.fill(&mut reader1).unwrap();

        // Second fill should preserve look-ahead
        let data2 = b"Second chunk.";
        let mut reader2 = Cursor::new(data2);
        let bytes_read = buffer.fill(&mut reader2).unwrap();

        // Should have look-ahead from first chunk + new data
        assert!(bytes_read > data2.len());
    }
}
