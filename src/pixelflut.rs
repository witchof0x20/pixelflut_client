use std::io;
use std::net::SocketAddr;

use futures::{self, Future, Sink, Stream};
use tokio::codec::Framed;
use tokio::codec::LinesCodec;
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct PixelFlut {
    stream: Framed<TcpStream, LinesCodec>,
    width: Option<usize>,
    height: Option<usize>,
}

impl PixelFlut {
    pub fn connect(addr: &SocketAddr) -> impl Future<Item = Self, Error = io::Error> {
        // Connect to the given address
        TcpStream::connect(addr).map(|stream| PixelFlut {
            stream: Framed::new(stream, LinesCodec::new()),
            width: None,
            height: None,
        })
    }
    pub fn with_size(self) -> impl Future<Item = Self, Error = io::Error> {
        self.stream
            // Send the size command
            .send(String::from("SIZE"))
            // Get the next line as a response
            .and_then(|stream| stream.into_future().map_err(|(err, _)| err))
            // This returns an option, so flatten it out
            .and_then(|(response, stream)| {
                response.map(|response| (response, stream)).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "Failed to receive respnse")
                })
            })
            // Handle the response
            .and_then(|(response, stream)| {
                // Split up the response and skip the first token, which is SIZE
                // TODO: consider checking this token
                let mut tokens = response.split(' ').skip(1);
                // Get the width
                let width = tokens
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::Other, "Size response missing width")
                    })?
                    .parse()
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
                // Get the height
                let height = tokens
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::Other, "Size response missing height")
                    })?
                    .parse()
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
                // Use the width, height, and stream to recreate the pixelflut controller with
                // dimensions added
                Ok(PixelFlut {
                    stream,
                    width: Some(width),
                    height: Some(height),
                })
            })
    }
    pub fn send_pixels<I>(self, pixels: I) -> impl Future<Item = Self, Error = io::Error>
    where
        I: IntoIterator<Item = String>,
    {
        let width = self.width;
        let height = self.height;
        self.stream
            .send_all(futures::stream::iter_ok::<_, io::Error>(pixels))
            .map(move |(stream, _)| PixelFlut {
                stream,
                width,
                height,
            })
    }
}
