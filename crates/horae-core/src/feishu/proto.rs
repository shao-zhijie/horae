//! 飞书 WebSocket 长连接 Protobuf 二进制帧协议定义。
//!
//! 飞书开放平台长连接网关采用 Protobuf 编码二进制数据帧。
//! 本模块采用零外部依赖的纯手写 Protobuf Wire Format 编解码实现，
//! 无需依赖 `prost` / `prost-derive` 或外部 `protoc` 编译器。

use std::fmt;

/// Protobuf 解码错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError(&'static str);

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for DecodeError {}

/// 帧头部键值对
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Header {
    pub key: String,
    pub value: String,
}

impl Header {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    fn encoded_len(&self) -> usize {
        let mut len = 0;
        if !self.key.is_empty() {
            len += 1 + varint_len(self.key.len() as u64) + self.key.len();
        }
        if !self.value.is_empty() {
            len += 1 + varint_len(self.value.len() as u64) + self.value.len();
        }
        len
    }

    fn encode_into(&self, buf: &mut Vec<u8>) {
        if !self.key.is_empty() {
            encode_bytes_field(1, self.key.as_bytes(), buf);
        }
        if !self.value.is_empty() {
            encode_bytes_field(2, self.value.as_bytes(), buf);
        }
    }

    fn decode(mut buf: &[u8]) -> Result<Self, DecodeError> {
        let mut header = Self::default();
        while !buf.is_empty() {
            let tag = read_varint(&mut buf)?;
            let field_num = tag >> 3;
            let wire_type = (tag & 0x07) as u8;
            match (field_num, wire_type) {
                (1, 2) => header.key = read_string(&mut buf)?,
                (2, 2) => header.value = read_string(&mut buf)?,
                _ => skip_field(wire_type, &mut buf)?,
            }
        }
        Ok(header)
    }
}

/// 飞书长连接二进制交互单元 Frame
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    pub seq_id: u64,
    pub log_id: u64,
    pub service: i32,
    pub method: i32,
    pub headers: Vec<Header>,
    pub payload_encoding: Option<String>,
    pub payload_type: Option<String>,
    pub payload: Option<Vec<u8>>,
    pub log_id_new: Option<String>,
}

/// 帧方法类型（与飞书官方网关一致）
pub const METHOD_CONTROL: i32 = 0;
pub const METHOD_DATA: i32 = 1;

/// 消息类型常量
pub const MSG_TYPE_PING: &str = "ping";
pub const MSG_TYPE_PONG: &str = "pong";
pub const MSG_TYPE_EVENT: &str = "event";
pub const MSG_TYPE_ACK: &str = "ack";

impl Frame {
    /// 编码为 Protobuf 二进制字节流
    pub fn encode_to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64 + self.payload.as_ref().map_or(0, Vec::len));
        if self.seq_id != 0 {
            encode_varint_field(1, self.seq_id, &mut buf);
        }
        if self.log_id != 0 {
            encode_varint_field(2, self.log_id, &mut buf);
        }
        if self.service != 0 {
            encode_varint_field(3, self.service as i64 as u64, &mut buf);
        }
        if self.method != 0 {
            encode_varint_field(4, self.method as i64 as u64, &mut buf);
        }
        for h in &self.headers {
            let sub_len = h.encoded_len();
            encode_varint((5 << 3) | 2, &mut buf);
            encode_varint(sub_len as u64, &mut buf);
            h.encode_into(&mut buf);
        }
        if let Some(ref s) = self.payload_encoding {
            encode_bytes_field(6, s.as_bytes(), &mut buf);
        }
        if let Some(ref s) = self.payload_type {
            encode_bytes_field(7, s.as_bytes(), &mut buf);
        }
        if let Some(ref b) = self.payload {
            encode_bytes_field(8, b, &mut buf);
        }
        if let Some(ref s) = self.log_id_new {
            encode_bytes_field(9, s.as_bytes(), &mut buf);
        }
        buf
    }

    /// 从 Protobuf 二进制字节流解码
    pub fn decode(mut buf: &[u8]) -> Result<Self, DecodeError> {
        let mut frame = Self::default();
        while !buf.is_empty() {
            let tag = read_varint(&mut buf)?;
            let field_num = tag >> 3;
            let wire_type = (tag & 0x07) as u8;
            match (field_num, wire_type) {
                (1, 0) => frame.seq_id = read_varint(&mut buf)?,
                (2, 0) => frame.log_id = read_varint(&mut buf)?,
                (3, 0) => frame.service = read_varint(&mut buf)? as i32,
                (4, 0) => frame.method = read_varint(&mut buf)? as i32,
                (5, 2) => {
                    let bytes = read_bytes(&mut buf)?;
                    frame.headers.push(Header::decode(bytes)?);
                }
                (6, 2) => frame.payload_encoding = Some(read_string(&mut buf)?),
                (7, 2) => frame.payload_type = Some(read_string(&mut buf)?),
                (8, 2) => frame.payload = Some(read_bytes(&mut buf)?.to_vec()),
                (9, 2) => frame.log_id_new = Some(read_string(&mut buf)?),
                _ => skip_field(wire_type, &mut buf)?,
            }
        }
        Ok(frame)
    }

    /// 获取指定 header 键的值
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|h| h.key.eq_ignore_ascii_case(key))
            .map(|h| h.value.as_str())
    }

    /// 判断当前帧是否为网关 Ping 心跳帧
    pub fn is_ping(&self) -> bool {
        self.method == METHOD_CONTROL && self.header("type") == Some(MSG_TYPE_PING)
    }

    /// 构造客户端主动发送的 Ping 心跳保活帧（用于维持长连接在线状态）
    pub fn build_ping(service_id: i32) -> Self {
        Self {
            seq_id: 0,
            log_id: 0,
            service: service_id,
            method: METHOD_CONTROL,
            headers: vec![Header::new("type", MSG_TYPE_PING)],
            payload_encoding: None,
            payload_type: None,
            payload: None,
            log_id_new: None,
        }
    }

    /// 构造网关 Pong 心跳响应帧
    pub fn build_pong(seq_id: u64, service: i32) -> Self {
        Self {
            seq_id,
            log_id: 0,
            service,
            method: METHOD_CONTROL,
            headers: vec![Header::new("type", MSG_TYPE_PONG)],
            payload_encoding: None,
            payload_type: None,
            payload: None,
            log_id_new: None,
        }
    }

    /// 构造事件消费成功或卡片回调的 ACK 响应帧
    pub fn build_ack(seq_id: u64, service: i32, method: i32, ack_payload: Option<Vec<u8>>) -> Self {
        Self {
            seq_id,
            log_id: 0,
            service,
            method,
            headers: vec![Header::new("type", MSG_TYPE_ACK)],
            payload_encoding: None,
            payload_type: None,
            payload: ack_payload,
            log_id_new: None,
        }
    }
}

#[inline]
fn varint_len(mut v: u64) -> usize {
    let mut n = 1;
    while v >= 0x80 {
        v >>= 7;
        n += 1;
    }
    n
}

#[inline]
fn encode_varint(mut v: u64, buf: &mut Vec<u8>) {
    while v >= 0x80 {
        buf.push((v as u8) | 0x80);
        v >>= 7;
    }
    buf.push(v as u8);
}

#[inline]
fn encode_varint_field(field_num: u64, val: u64, buf: &mut Vec<u8>) {
    encode_varint(field_num << 3, buf);
    encode_varint(val, buf);
}

#[inline]
fn encode_bytes_field(field_num: u64, bytes: &[u8], buf: &mut Vec<u8>) {
    encode_varint((field_num << 3) | 2, buf);
    encode_varint(bytes.len() as u64, buf);
    buf.extend_from_slice(bytes);
}

#[inline]
fn read_varint(buf: &mut &[u8]) -> Result<u64, DecodeError> {
    let mut result = 0u64;
    let mut shift = 0u32;
    for _ in 0..10 {
        let Some((&byte, rest)) = buf.split_first() else {
            return Err(DecodeError("unexpected EOF reading varint"));
        };
        *buf = rest;
        result |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
    }
    Err(DecodeError("varint overflow"))
}

#[inline]
fn read_bytes<'a>(buf: &mut &'a [u8]) -> Result<&'a [u8], DecodeError> {
    let len = read_varint(buf)? as usize;
    if buf.len() < len {
        return Err(DecodeError("unexpected EOF reading length-delimited bytes"));
    }
    let (head, tail) = buf.split_at(len);
    *buf = tail;
    Ok(head)
}

#[inline]
fn read_string(buf: &mut &[u8]) -> Result<String, DecodeError> {
    let bytes = read_bytes(buf)?;
    std::str::from_utf8(bytes)
        .map(String::from)
        .map_err(|_| DecodeError("invalid UTF-8 in protobuf string"))
}

fn skip_field(wire_type: u8, buf: &mut &[u8]) -> Result<(), DecodeError> {
    match wire_type {
        0 => {
            let _ = read_varint(buf)?;
            Ok(())
        }
        1 => {
            if buf.len() < 8 {
                return Err(DecodeError("unexpected EOF skipping 64-bit field"));
            }
            *buf = &buf[8..];
            Ok(())
        }
        2 => {
            let _ = read_bytes(buf)?;
            Ok(())
        }
        5 => {
            if buf.len() < 4 {
                return Err(DecodeError("unexpected EOF skipping 32-bit field"));
            }
            *buf = &buf[4..];
            Ok(())
        }
        _ => Err(DecodeError("unsupported protobuf wire type")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_encode_decode() {
        let frame = Frame {
            seq_id: 12345,
            log_id: 67890,
            service: 1,
            method: METHOD_DATA,
            headers: vec![
                Header::new("type", "event"),
                Header::new("message_id", "msg-001"),
            ],
            payload_encoding: None,
            payload_type: Some("json".into()),
            payload: Some(b"{\"hello\":\"world\"}".to_vec()),
            log_id_new: None,
        };

        let encoded = frame.encode_to_vec();
        assert!(!encoded.is_empty());

        let decoded = Frame::decode(&encoded[..]).expect("decode frame");
        assert_eq!(decoded.seq_id, 12345);
        assert_eq!(decoded.header("type"), Some("event"));
        assert_eq!(decoded.header("message_id"), Some("msg-001"));
        assert_eq!(decoded.header("non_existent"), None);
        assert_eq!(
            decoded.payload.as_deref(),
            Some(b"{\"hello\":\"world\"}".as_slice())
        );
    }

    #[test]
    fn test_ping_pong_frames() {
        let ping = Frame {
            seq_id: 888,
            log_id: 0,
            service: 1,
            method: METHOD_CONTROL,
            headers: vec![Header::new("type", "ping")],
            payload_encoding: None,
            payload_type: None,
            payload: None,
            log_id_new: None,
        };
        assert!(ping.is_ping());

        let pong = Frame::build_pong(ping.seq_id, ping.service);
        assert_eq!(pong.seq_id, 888);
        assert_eq!(pong.method, METHOD_CONTROL);
        assert_eq!(pong.header("type"), Some("pong"));
    }
}
