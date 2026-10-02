use std::fmt;

#[derive(Debug)]
enum EncodeError {
    TooLong(usize),
    NotFinite(usize),
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::TooLong(n) => write!(f, "{n} samples do not fit in a wav chunk"),
            EncodeError::NotFinite(i) => write!(f, "sample {i} is not a finite number"),
        }
    }
}

impl std::error::Error for EncodeError {}

fn to_pcm16(samples: &[f32]) -> Result<Vec<i16>, EncodeError> {
    samples
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            if !s.is_finite() {
                return Err(EncodeError::NotFinite(i));
            }
            let clamped = s.clamp(-1.0, 1.0);
            Ok((clamped * i16::MAX as f32).round() as i16)
        })
        .collect()
}

fn header(pcm: &[i16], sample_rate: u32) -> Result<Vec<u8>, EncodeError> {
    let too_long = || EncodeError::TooLong(pcm.len());
    let data_len = u32::try_from(pcm.len() * 2).map_err(|_| too_long())?;
    let riff_len = data_len.checked_add(36).ok_or_else(too_long)?;
    let byte_rate = sample_rate.checked_mul(2).ok_or_else(too_long)?;
    let mut out = Vec::with_capacity(44);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_len.to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    Ok(out)
}

fn peak(pcm: &[i16]) -> Option<i32> {
    pcm.iter().map(|&s| i32::from(s).abs()).max()
}

fn main() -> Result<(), EncodeError> {
    let samples: Vec<f32> = (0..32u16)
        .map(|i| (f32::from(i) / 4.0).sin() * 1.3)
        .collect();
    let pcm = to_pcm16(&samples)?;
    let head = header(&pcm, 44_100)?;
    println!("header is {} bytes", head.len());
    println!("first samples {:?}", &pcm[..6]);
    println!("peak amplitude {:?}", peak(&pcm));
    match to_pcm16(&[0.5, f32::NAN]) {
        Ok(_) => println!("unexpected success"),
        Err(e) => println!("rejected: {e}"),
    }
    Ok(())
}
