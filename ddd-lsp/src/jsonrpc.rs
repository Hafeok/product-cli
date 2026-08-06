//! Content-Length framing for LSP JSON-RPC byte streams.
//!
//! LSP messages travel as `Content-Length: N\r\n\r\n<N bytes of JSON>`.
//! Both the client and the mock host speak through these two functions, so
//! a framing bug cannot hide on one side of the tests.

use std::io::{BufRead, Write};

use serde_json::Value;

/// Write one framed message.
pub fn write_message<W: Write>(w: &mut W, v: &Value) -> std::io::Result<()> {
    let body = serde_json::to_vec(v)?;
    write!(w, "Content-Length: {}\r\n\r\n", body.len())?;
    w.write_all(&body)?;
    w.flush()
}

/// Read one framed message; `None` on a clean EOF at a frame boundary.
pub fn read_message<R: BufRead>(r: &mut R) -> std::io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = r.read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(rest) = header_value(line, "Content-Length") {
            content_length = rest.trim().parse::<usize>().ok();
        }
    }
    let len = content_length.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "missing Content-Length header")
    })?;
    let mut body = vec![0u8; len];
    r.read_exact(&mut body)?;
    let v = serde_json::from_slice(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(Some(v))
}

/// Case-insensitive `Header: value` match.
fn header_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let (key, value) = line.split_once(':')?;
    if key.trim().eq_ignore_ascii_case(name) {
        Some(value)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn round_trips_a_message() {
        let msg = json!({"jsonrpc": "2.0", "id": 1, "method": "x", "params": {"a": "ü"}});
        let mut buf = Vec::new();
        write_message(&mut buf, &msg).expect("write");
        let mut r = std::io::BufReader::new(&buf[..]);
        let back = read_message(&mut r).expect("read").expect("some");
        assert_eq!(back, msg);
        assert!(read_message(&mut r).expect("eof").is_none());
    }

    #[test]
    fn reads_two_consecutive_frames() {
        let (a, b) = (json!({"id": 1}), json!({"id": 2}));
        let mut buf = Vec::new();
        write_message(&mut buf, &a).expect("write a");
        write_message(&mut buf, &b).expect("write b");
        let mut r = std::io::BufReader::new(&buf[..]);
        assert_eq!(read_message(&mut r).expect("a").expect("some"), a);
        assert_eq!(read_message(&mut r).expect("b").expect("some"), b);
    }

    #[test]
    fn missing_length_is_an_error() {
        let raw = b"X-Other: 1\r\n\r\n{}";
        let mut r = std::io::BufReader::new(&raw[..]);
        assert!(read_message(&mut r).is_err());
    }
}
