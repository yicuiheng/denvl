use crate::communicate::error::Result;
use crate::communicate::named_pipe_pair::Stream;

pub enum Message {
    Request(Request),
    Response(Response),
    Notif(Notif),
}
pub struct Request {
    pub id: u64,
    pub method: String,
    pub params: Vec<serde_json::Value>,
}

pub struct Response {
    pub id: Option<u64>,
    pub result: serde_json::Value,
    pub error: Option<ResponseError>,
}

pub struct ResponseError {
    pub code: u64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

pub struct Notif {
    pub method: String,
    pub params: serde_json::Value,
}

pub fn read(stream: &mut Stream) -> Result<serde_json::Value> {
    use std::io::Read;
    let mut len: Option<u64> = None;
    let mut result = String::new();

    // read length
    loop {
        let mut line = readline(stream)?;
        if line == "\r\n" {
            break;
        }
        line.make_ascii_lowercase();
        const CONTENT_LENGTH: &str = "content-length: ";
        if line.starts_with(CONTENT_LENGTH) {
            let line: String = line.chars().skip(CONTENT_LENGTH.len()).collect();
            len = Some(line.trim().parse().unwrap());
        }
    }

    // read body
    let reader = stream.input.take(len.unwrap());
    let v = serde_json::from_reader(reader)?;
    Ok(v)
}

fn readline(stream: &mut Stream) -> Result<String> {
    use std::io::Read;
    let mut result = String::new();
    loop {
        let mut c: [char; 1] = [' '];
        unsafe {
            stream
                .input
                .read_exact(std::mem::transmute(c.as_mut_slice()))?;
        }
        result.push(c[0]);

        if c[0] == '\n' {
            break;
        }
    }
    Ok(result)
}

pub fn write(stream: &mut Stream, value: serde_json::Value) -> Result<()> {
    use std::io::Write;
    let body = serde_json::to_vec(&value)?;
    stream.output.write_all(b"Content-Length: ")?;
    stream.output.write_all(body.len().to_string().as_bytes())?;
    stream.output.write_all(b"\r\n\r\n")?;
    stream.output.write_all(&body)?;
    Ok(())
}
