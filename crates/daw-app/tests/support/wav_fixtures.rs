pub fn chunk(id: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut result = id.to_vec();
    result.extend((body.len() as u32).to_le_bytes());
    result.extend(body);
    if !body.len().is_multiple_of(2) {
        result.push(0);
    }
    result
}
pub fn riff(chunks: &[Vec<u8>]) -> Vec<u8> {
    let mut result = b"RIFF".to_vec();
    result.extend((4 + chunks.iter().map(Vec::len).sum::<usize>() as u32).to_le_bytes());
    result.extend(b"WAVE");
    for chunk in chunks {
        result.extend(chunk);
    }
    result
}
pub fn format(tag: u16, bits: u16, channels: u16, rate: u32, extensible: bool) -> Vec<u8> {
    let width = channels * (bits / 8);
    let mut result = (if extensible { 0xfffeu16 } else { tag })
        .to_le_bytes()
        .to_vec();
    result.extend(channels.to_le_bytes());
    result.extend(rate.to_le_bytes());
    result.extend(rate.wrapping_mul(u32::from(width)).to_le_bytes());
    result.extend(width.to_le_bytes());
    result.extend(bits.to_le_bytes());
    if extensible {
        result.extend(22u16.to_le_bytes());
        result.extend(bits.to_le_bytes());
        result.extend((if channels == 1 { 1u32 } else { 3 }).to_le_bytes());
        result.extend(u32::from(tag).to_le_bytes());
        result.extend([0, 0, 0x10, 0, 0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71]);
    }
    result
}
pub fn wav(tag: u16, bits: u16, channels: u16, data: &[u8], extensible: bool) -> Vec<u8> {
    riff(&[
        chunk(b"fmt ", &format(tag, bits, channels, 48_000, extensible)),
        chunk(b"data", data),
    ])
}
