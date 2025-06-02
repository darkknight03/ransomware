use std::io::{Cursor, Error as IoError, ErrorKind};
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};
use serde::{Serialize, de::DeserializeOwned};
use serde_json;

/// A codec that handles sending and receiving JSON messages over TCP,
/// framed by a u64 length prefix (8 bytes).
///
/// Each message is sent as:
/// [8-byte length prefix][JSON-encoded message]
///
/// This keeps message boundaries clear, even over a continuous stream like TCP.
pub struct JsonCodec<T> {
    // This PhantomData tells the compiler we depend on T,
    // even though we don’t store an actual T inside this struct.
    _phantom: std::marker::PhantomData<T>,
}

impl<T> JsonCodec<T> {
    /// Create a new JSON codec for the given type T.
    pub fn new() -> Self {
        JsonCodec {
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Decoder implementation: Converts raw TCP bytes → structured Rust messages.
impl<T> Decoder for JsonCodec<T>
where
    T: DeserializeOwned, // T must be deserializable from JSON
{
    type Item = T;
    type Error = IoError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, IoError> {
        // Check if we have enough bytes for the 8-byte length prefix
        if src.len() < 8 {
            return Ok(None); // Wait for more data
        }

        // Use Cursor to peek at the length prefix without consuming bytes yet
        let mut cursor = Cursor::new(&src[..]);
        let msg_len = cursor.get_u64() as usize;

        // Wait until we have the full message payload in the buffer
        if src.len() < 8 + msg_len {
            return Ok(None); // Not enough data yet
        }

        // Advance buffer by 8 bytes to drop the length prefix
        src.advance(8);

        // Extract the exact number of bytes for the JSON message
        let json_bytes = src.split_to(msg_len);

        // Try to deserialize the JSON bytes into type T
        match serde_json::from_slice::<T>(&json_bytes) {
            Ok(msg) => Ok(Some(msg)),
            Err(err) => Err(IoError::new(ErrorKind::InvalidData, err)),
        }
    }
}

/// Encoder implementation: Converts structured Rust messages → raw TCP bytes.
impl<T> Encoder<T> for JsonCodec<T>
where
    T: Serialize, // T must be serializable to JSON
{
    type Error = IoError;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), IoError> {
        // Serialize the message into JSON bytes
        let json = serde_json::to_vec(&item)
            .map_err(|e| IoError::new(ErrorKind::InvalidInput, e))?;

        let len = json.len() as u64;

        // Reserve space for [length prefix] + [JSON]
        dst.reserve(8 + json.len());

        // Write the 8-byte length prefix
        dst.put_u64(len);

        // Write the JSON-encoded bytes
        dst.extend_from_slice(&json);

        Ok(())
    }
}
