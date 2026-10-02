use std::fmt;

#[derive(Debug, PartialEq)]
enum TlvError {
    ValueTooLarge { tag: u8, len: usize },
    Truncated { at: usize },
}

impl fmt::Display for TlvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TlvError::ValueTooLarge { tag, len } => write!(f, "tag {tag}: {len} bytes exceeds limit"),
            TlvError::Truncated { at } => write!(f, "record truncated at byte {at}"),
        }
    }
}

fn encode(records: &[(u8, &[u8])]) -> Result<Vec<u8>, TlvError> {
    let mut out = Vec::new();
    for &(tag, value) in records {
        let len = u16::try_from(value.len())
            .map_err(|_| TlvError::ValueTooLarge { tag, len: value.len() })?;
        out.push(tag);
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(value);
    }
    Ok(out)
}

fn decode(mut buf: &[u8]) -> Result<Vec<(u8, Vec<u8>)>, TlvError> {
    let start = buf.len();
    let mut records = Vec::new();
    while let Some((&tag, rest)) = buf.split_first() {
        let at = start - buf.len();
        let (len_bytes, rest) = rest.split_at_checked(2).ok_or(TlvError::Truncated { at })?;
        let len = u16::from_be_bytes([len_bytes[0], len_bytes[1]]) as usize;
        let (value, rest) = rest.split_at_checked(len).ok_or(TlvError::Truncated { at })?;
        records.push((tag, value.to_vec()));
        buf = rest;
    }
    Ok(records)
}

fn main() {
    let temperature = 2155i32.to_be_bytes();
    let humidity = 61u8.to_be_bytes();
    let station = b"north-ridge";
    let records: [(u8, &[u8]); 3] = [(1, &temperature), (2, &humidity), (3, station)];
    let bytes = match encode(&records) {
        Ok(b) => b,
        Err(e) => {
            println!("encode failed: {e}");
            return;
        }
    };
    println!("encoded {} bytes", bytes.len());
    match decode(&bytes) {
        Ok(decoded) => {
            for (tag, value) in decoded {
                println!("tag {tag}: {value:?}");
            }
        }
        Err(e) => println!("decode failed: {e}"),
    }
    let big = vec![0u8; 70_000];
    println!("{:?}", encode(&[(9, &big)]).map(|b| b.len()));
    println!("{:?}", decode(&bytes[..5]).map(|r| r.len()));
}
