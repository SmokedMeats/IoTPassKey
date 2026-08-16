use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use stealth_bonding_sim::fixture_sc_public;

pub const DEFAULT_ADDR: &str = "127.0.0.1:17420";
pub const HELLO_OPCODE: u8 = 0xFE;
pub const STATUS_OPCODE: u8 = 0xFD;
pub const HELLO_LEN: usize = 40;
pub const STATUS_LEN: usize = 3;
pub const STATUS_OK: u8 = 0;
pub const STATUS_COMMITMENT_FAILED: u8 = 1;
pub const STATUS_NO_BOND: u8 = 2;
pub const STATUS_ASSERTION_FAILED: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkHello {
    pub identity: [u8; 6],
}

impl LinkHello {
    pub fn encode(self) -> [u8; HELLO_LEN] {
        let mut out = [0u8; HELLO_LEN];
        out[0] = HELLO_OPCODE;
        out[1] = 0x01;
        out[2..8].copy_from_slice(&self.identity);
        out[8..40].copy_from_slice(&fixture_sc_public(self.identity));
        out
    }

    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() != HELLO_LEN {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "hello length"));
        }
        if bytes[0] != HELLO_OPCODE || bytes[1] != 0x01 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "hello header"));
        }
        let mut identity = [0u8; 6];
        identity.copy_from_slice(&bytes[2..8]);
        if bytes[8..40] != fixture_sc_public(identity) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "hello Secure Connections fixture mismatch",
            ));
        }
        Ok(Self { identity })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkStatus {
    pub code: u8,
}

impl LinkStatus {
    pub fn ok() -> Self {
        Self { code: STATUS_OK }
    }

    pub fn encode(self) -> [u8; STATUS_LEN] {
        [STATUS_OPCODE, 0x01, self.code]
    }

    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() != STATUS_LEN {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "status length"));
        }
        if bytes[0] != STATUS_OPCODE || bytes[1] != 0x01 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "status header"));
        }
        Ok(Self { code: bytes[2] })
    }
}

pub fn listen(addr: impl ToSocketAddrs) -> io::Result<TcpListener> {
    let listener = TcpListener::bind(addr)?;
    listener.set_nonblocking(false)?;
    Ok(listener)
}

pub fn accept(listener: &TcpListener) -> io::Result<TcpStream> {
    let (stream, _) = listener.accept()?;
    stream.set_nodelay(true)?;
    Ok(stream)
}

pub fn connect(addr: impl ToSocketAddrs) -> io::Result<TcpStream> {
    let stream = TcpStream::connect(addr)?;
    stream.set_nodelay(true)?;
    Ok(stream)
}

pub fn write_frame(stream: &mut TcpStream, payload: &[u8]) -> io::Result<()> {
    if payload.len() > u16::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "frame is too long",
        ));
    }
    stream.write_all(&(payload.len() as u16).to_be_bytes())?;
    stream.write_all(payload)?;
    stream.flush()
}

pub fn read_frame(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut len_bytes = [0u8; 2];
    stream.read_exact(&mut len_bytes)?;
    let len = u16::from_be_bytes(len_bytes) as usize;
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload)?;
    Ok(payload)
}
