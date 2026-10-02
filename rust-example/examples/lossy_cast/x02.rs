fn frame(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 1);
    out.push(payload.len() as u8);
    out.extend_from_slice(payload);
    out
}

fn unframe(buf: &[u8]) -> Option<&[u8]> {
    let (&len, rest) = buf.split_first()?;
    rest.get(..usize::from(len))
}

fn main() {
    let messages = ["ping", "status:ok", "a somewhat longer message body"];
    let mut stream = Vec::new();
    for m in messages {
        stream.extend(frame(m.as_bytes()));
    }
    let mut cursor = stream.as_slice();
    while let Some(body) = unframe(cursor) {
        println!("decoded {:?}", String::from_utf8_lossy(body));
        cursor = &cursor[1 + body.len()..];
    }
    println!("stream was {} bytes", stream.len());
}
