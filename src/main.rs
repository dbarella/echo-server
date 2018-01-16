extern crate bytes;
extern crate futures;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;
use std::str;
use bytes::BytesMut;
use futures::{future, Future};

use tokio_io::codec::Framed;
use tokio_io::codec::{Encoder, Decoder};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::TcpServer;
use tokio_proto::pipeline::ServerProto;
use tokio_service::Service;


pub struct LineCodec;


impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<String>> {
        if let Some(i) = buf.iter().position(|&b| b == b'\n') {
            // remove the serialized frame from the buffer.
            let line = buf.split_to(i);

            // remove the newline
            buf.split_to(1);

            // turn the data into a UTF string and return it as a Frame
            match str::from_utf8(&line) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other,
                                             "invalid UTF-8.")),
            }
        } else {
            Ok(None)
        }
    }
}


impl Encoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, msg: String, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(msg.as_bytes());
        buf.extend(b"\n");
        Ok(())
    }
}


pub struct LineProto;


impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for LineProto {
    // Request matches the Item type of the Decoder
    type Request = String;

    // Response matches the Item type of the Encoder
    type Response = String;

    // Boilerplate to hook in the codec
    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}


pub struct Hello;


impl Service for Hello {
    type Request = String;
    type Response = String;

    // Non-streaming protocol -> service errors are always io::Error-s
    type Error = io::Error;

    // Box the future for completing the response -- just for simplicity
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    // Produce a future for computing a response from a request
    fn call(&self, req: Self::Request) -> Self::Future {
        // Immediate future response
        Box::new(future::ok(req))
    }
}


pub struct Olleh;


impl Service for Olleh {
    type Request = String;
    type Response = String;

    // Non-streaming protocol -> service errors are always io::Error-s
    type Error = io::Error;

    // Box the future for completing the response -- just for simplicity
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    // Produce a future for computing a response from a request
    fn call(&self, req: Self::Request) -> Self::Future {
        let rev: String = req.chars()
            .rev()
            .collect();
        Box::new(future::ok(rev))
    }
}


fn main() {
    // We are local hosts
    let addr = "0.0.0.0:12345".parse().unwrap();

    // Builder requires a protocol and an address
    let server = TcpServer::new(LineProto, addr);

    // Immediately return a new instance of the server
    server.serve(|| Ok(Olleh));
}
