pub fn take<'a>(data: &'a [u8], at: usize, len: usize) -> Result<&'a [u8], String> {
    data.get(at..at + len)
        .ok_or_else(|| "arquivo truncado".to_owned())
}

pub fn u16_le(data: &[u8], at: usize) -> Result<u16, String> {
    let b = take(data, at, 2)?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}

pub fn u32_le(data: &[u8], at: usize) -> Result<u32, String> {
    let b = take(data, at, 4)?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

pub fn u16_be(data: &[u8], at: usize) -> Result<u16, String> {
    let b = take(data, at, 2)?;
    Ok(u16::from_be_bytes([b[0], b[1]]))
}

pub fn u32_be(data: &[u8], at: usize) -> Result<u32, String> {
    let b = take(data, at, 4)?;
    Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

pub fn i32_le(data: &[u8], at: usize) -> Result<i32, String> {
    Ok(u32_le(data, at)? as i32)
}
