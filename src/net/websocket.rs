use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use rustls::ClientConfig;
use std::sync::Arc;
use base64::Engine as _;
type BoxErr = Box<dyn std::error::Error + Send + Sync>;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WebSocketState { Connecting, Open, Closing, Closed }
#[derive(Debug, Clone)]
pub struct WebSocketFrame {
    pub opcode: u8,
    pub payload: Vec<u8>,
    pub fin: bool,
}
impl WebSocketFrame {
    pub const TEXT: u8 = 1;
    pub const BINARY: u8 = 2;
    pub const CLOSE: u8 = 8;
    pub const PING: u8 = 9;
    pub const PONG: u8 = 10;
    pub fn text(msg: &str) -> Self { Self { opcode: Self::TEXT, payload: msg.as_bytes().to_vec(), fin: true } }
    pub fn binary(data: &[u8]) -> Self { Self { opcode: Self::BINARY, payload: data.to_vec(), fin: true } }
    pub fn close() -> Self { Self { opcode: Self::CLOSE, payload: Vec::new(), fin: true } }
    pub fn ping() -> Self { Self { opcode: Self::PING, payload: Vec::new(), fin: true } }
    pub fn pong(payload: Vec<u8>) -> Self { Self { opcode: Self::PONG, payload, fin: true } }
    pub fn encode_masked(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let first = (if self.fin { 0x80 } else { 0 }) | (self.opcode & 0x0F);
        buf.push(first);
        let len = self.payload.len();
        if len < 126 {
            buf.push((len as u8) | 0x80);
        } else if len <= 65535 {
            buf.push(126 | 0x80);
            buf.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            buf.push(127 | 0x80);
            buf.extend_from_slice(&(len as u64).to_be_bytes());
        }
        let mask_key: [u8; 4] = rand::random();
        buf.extend_from_slice(&mask_key);
        for (i, &b) in self.payload.iter().enumerate() {
            buf.push(b ^ mask_key[i % 4]);
        }
        buf
    }
    pub fn encode_unmasked(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let first = (if self.fin { 0x80 } else { 0 }) | (self.opcode & 0x0F);
        buf.push(first);
        let len = self.payload.len();
        if len < 126 {
            buf.push(len as u8);
        } else if len <= 65535 {
            buf.push(126);
            buf.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            buf.push(127);
            buf.extend_from_slice(&(len as u64).to_be_bytes());
        }
        buf.extend_from_slice(&self.payload);
        buf
    }
}
pub async fn decode_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<WebSocketFrame, BoxErr> {
    let first = reader.read_u8().await?;
    let fin = (first & 0x80) != 0;
    let opcode = first & 0x0F;
    let second = reader.read_u8().await?;
    let masked = (second & 0x80) != 0;
    let len1 = (second & 0x7F) as u64;
    let payload_len = if len1 < 126 {
        len1 as usize
    } else if len1 == 126 {
        let mut b = [0u8; 2];
        reader.read_exact(&mut b).await?;
        u16::from_be_bytes(b) as usize
    } else {
        let mut b = [0u8; 8];
        reader.read_exact(&mut b).await?;
        u64::from_be_bytes(b) as usize
    };
    let mask_key = if masked {
        let mut mk = [0u8; 4];
        reader.read_exact(&mut mk).await?;
        Some(mk)
    } else { None };
    let mut payload = vec![0u8; payload_len];
    if payload_len > 0 { reader.read_exact(&mut payload).await?; }
    if let Some(mk) = mask_key {
        for (i, b) in payload.iter_mut().enumerate() { *b ^= mk[i % 4]; }
    }
    Ok(WebSocketFrame { opcode, payload, fin })
}
fn parse_ws_url(url: &str) -> Result<(String, u16, String, bool), BoxErr> {
    let secure = url.starts_with("wss://");
    let stripped = url.strip_prefix("wss://").or_else(|| url.strip_prefix("ws://"))
        .ok_or("invalid ws url")?;
    let (host_port, path) = stripped.split_once('/').map(|(h, p)| (h, format!("/{}", p)))
        .unwrap_or((stripped, "/".to_string()));
    let (host, port) = if let Some((h, p)) = host_port.split_once(':') {
        (h.to_string(), p.parse::<u16>()?)
    } else {
        (host_port.to_string(), if secure { 443 } else { 80 })
    };
    Ok((host, port, path, secure))
}
fn generate_ws_key() -> String {
    let bytes: [u8; 16] = rand::random();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
pub struct WebSocket {
    pub url: String,
    pub state: WebSocketState,
    pub sender: mpsc::Sender<WebSocketFrame>,
    pub receiver: mpsc::Receiver<WebSocketFrame>,
}
impl WebSocket {
    pub async fn connect(url: &str) -> Result<Self, BoxErr> {
        let (host, port, path, secure) = parse_ws_url(url)?;
        let addr = format!("{}:{}", host, port);
        let tcp = TcpStream::connect(&addr).await?;
        let ws_key = generate_ws_key();
        let handshake = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {}\r\nSec-WebSocket-Version: 13\r\n\r\n",
            path, host, ws_key
        );
        let (tx_out, mut rx_out) = mpsc::channel::<WebSocketFrame>(64);
        let (tx_in, rx_in) = mpsc::channel::<WebSocketFrame>(64);
        if secure {
            let mut root_store = rustls::RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let tls_config = ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();
            let connector = tokio_rustls::TlsConnector::from(Arc::new(tls_config));
            let server_name = rustls::pki_types::ServerName::try_from(host.clone())?;
            let mut tls_stream = connector.connect(server_name, tcp).await?;
            tls_stream.write_all(handshake.as_bytes()).await?;
            let mut resp_buf = vec![0u8; 4096];
            let n = tls_stream.read(&mut resp_buf).await?;
            let resp_str = String::from_utf8_lossy(&resp_buf[..n]);
            if !resp_str.contains("101") { return Err(format!("upgrade failed: {}", resp_str).into()); }
            let (mut read_half, mut write_half) = tokio::io::split(tls_stream);
            let tx_in_clone = tx_in.clone();
            tokio::spawn(async move {
                loop {
                    match decode_frame(&mut read_half).await {
                        Ok(frame) => {
                            if frame.opcode == WebSocketFrame::PING {
                                let _ = tx_in_clone.send(WebSocketFrame::pong(frame.payload)).await;
                                continue;
                            }
                            if frame.opcode == WebSocketFrame::CLOSE { break; }
                            if tx_in_clone.send(frame).await.is_err() { break; }
                        }
                        Err(_) => break,
                    }
                }
            });
            tokio::spawn(async move {
                while let Some(frame) = rx_out.recv().await {
                    let data = frame.encode_masked();
                    if write_half.write_all(&data).await.is_err() { break; }
                }
            });
        } else {
            let (mut read_half, mut write_half) = tokio::io::split(tcp);
            write_half.write_all(handshake.as_bytes()).await?;
            let mut resp_buf = vec![0u8; 4096];
            let n = read_half.read(&mut resp_buf).await?;
            let resp_str = String::from_utf8_lossy(&resp_buf[..n]);
            if !resp_str.contains("101") { return Err(format!("upgrade failed: {}", resp_str).into()); }
            let tx_in_clone = tx_in.clone();
            tokio::spawn(async move {
                loop {
                    match decode_frame(&mut read_half).await {
                        Ok(frame) => {
                            if frame.opcode == WebSocketFrame::PING {
                                let _ = tx_in_clone.send(WebSocketFrame::pong(frame.payload)).await;
                                continue;
                            }
                            if frame.opcode == WebSocketFrame::CLOSE { break; }
                            if tx_in_clone.send(frame).await.is_err() { break; }
                        }
                        Err(_) => break,
                    }
                }
            });
            tokio::spawn(async move {
                while let Some(frame) = rx_out.recv().await {
                    let data = frame.encode_masked();
                    if write_half.write_all(&data).await.is_err() { break; }
                }
            });
        }
        Ok(Self { url: url.to_string(), state: WebSocketState::Open, sender: tx_out, receiver: rx_in })
    }
    pub async fn send_text(&self, msg: &str) -> Result<(), BoxErr> {
        self.sender.send(WebSocketFrame::text(msg)).await.map_err(|e| e.to_string())?;
        Ok(())
    }
    pub async fn send_binary(&self, data: &[u8]) -> Result<(), BoxErr> {
        self.sender.send(WebSocketFrame::binary(data)).await.map_err(|e| e.to_string())?;
        Ok(())
    }
    pub async fn send_ping(&self) -> Result<(), BoxErr> {
        self.sender.send(WebSocketFrame::ping()).await.map_err(|e| e.to_string())?;
        Ok(())
    }
    pub async fn close(&mut self) -> Result<(), BoxErr> {
        self.state = WebSocketState::Closing;
        self.sender.send(WebSocketFrame::close()).await.map_err(|e| e.to_string())?;
        self.state = WebSocketState::Closed;
        Ok(())
    }
    pub async fn recv(&mut self) -> Option<WebSocketFrame> {
        self.receiver.recv().await
    }
}
pub struct WebSocketManager {
    connections: HashMap<String, WebSocket>,
}
impl WebSocketManager {
    pub fn new() -> Self { Self { connections: HashMap::new() } }
    pub async fn create(&mut self, id: &str, url: &str) -> Result<(), BoxErr> {
        let ws = WebSocket::connect(url).await?;
        self.connections.insert(id.to_string(), ws);
        Ok(())
    }
    pub async fn close(&mut self, id: &str) -> Result<(), BoxErr> {
        if let Some(ws) = self.connections.get_mut(id) {
            ws.close().await?;
        }
        self.connections.remove(id);
        Ok(())
    }
    pub async fn send(&self, id: &str, msg: &str) -> Result<(), BoxErr> {
        let ws = self.connections.get(id).ok_or("connection not found")?;
        ws.send_text(msg).await
    }
    pub async fn recv(&mut self, id: &str) -> Option<WebSocketFrame> {
        let ws = self.connections.get_mut(id)?;
        ws.recv().await
    }
    pub fn state(&self, id: &str) -> Option<WebSocketState> {
        self.connections.get(id).map(|ws| ws.state)
    }
    pub fn ids(&self) -> Vec<String> { self.connections.keys().cloned().collect() }
}
