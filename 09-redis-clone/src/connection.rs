#![allow(dead_code, unused_variables)]

use std::io::{self, Cursor};

use bytes::{Buf, BytesMut};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::{io::AsyncReadExt, net::TcpStream};

use mini_redis::frame::Error::Incomplete;
use mini_redis::{Frame, Result as RedisResult};

struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    // Read a frame from the connection
    pub fn new(stream: TcpStream) -> Self {
        // Allocate buffer of 4K
        let stream = BufWriter::new(stream);
        let buffer = BytesMut::with_capacity(4096);
        Connection { stream, buffer }
    }

    /// Read a frame from the connection
    pub async fn read_frame(&mut self) -> RedisResult<Option<Frame>> {
        loop {
            // Check if a frame can be built from the current buffer state
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // Try to read more data from the stream
            // If 0 bytes read, check if the buffer has data for clean shutdown
            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                // If no data on buffer, the client closed cleanly.
                // Else, the connection was interrupted.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    /// Write a frame to the connection
    pub async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
        match frame {
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*val).await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) => {
                let len = val.len();

                self.stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(_val) => unimplemented!(),
        }

        self.stream.flush().await?;

        Ok(())
    }

    /// Write a decimal frame to the stream
    async fn write_decimal(&mut self, val: u64) -> io::Result<()> {
        use std::io::Write;

        // Convert the value to a string
        let mut buf = [0u8; 12];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{}", val)?;

        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        self.stream.write_all(b"\r\n").await?;

        Ok(())
    }

    /// Try to parse a frame from the buffer. Empty it if successful.
    fn parse_frame(&mut self) -> RedisResult<Option<Frame>> {
        // Create a T:Buf type
        let mut buf = Cursor::new(&self.buffer[..]);

        // Check if a frame can be parsed from the buffer
        match Frame::check(&mut buf) {
            Ok(_) => {
                // Get the len of the frame (position of the cursor)
                let len = buf.position() as usize;

                // Reset the internal cursor
                buf.set_position(0);

                // Parse the frame
                let frame = Frame::parse(&mut buf)?;

                // Discard the frame from the buffer
                self.buffer.advance(len);

                // Return the frame
                Ok(Some(frame))
            }
            // Not enough data to parse, or an error has occurred
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
