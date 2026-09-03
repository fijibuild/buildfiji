//! LEB128 varints and delta-coded sorted id lists. Pure functions, so they
//! are Kani candidates later (bead buildfiji-2h9.8).

pub fn put_varint(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push((v as u8) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

pub fn get_varint(buf: &[u8], pos: &mut usize) -> Option<u64> {
    let mut v: u64 = 0;
    let mut shift = 0u32;
    loop {
        let b = *buf.get(*pos)?;
        *pos += 1;
        v |= u64::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            return Some(v);
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
}

pub fn put_u32s(out: &mut Vec<u8>, ids: &[u32]) {
    put_varint(out, ids.len() as u64);
    for &i in ids {
        put_varint(out, u64::from(i));
    }
}

pub fn get_u32s(buf: &[u8], pos: &mut usize) -> Option<Vec<u32>> {
    let n = get_varint(buf, pos)?;
    let mut v = Vec::with_capacity(n as usize);
    for _ in 0..n {
        v.push(u32::try_from(get_varint(buf, pos)?).ok()?);
    }
    Some(v)
}

/// Sorted ids as count + first + successive deltas.
pub fn put_deltas(out: &mut Vec<u8>, sorted: &[u32]) {
    put_varint(out, sorted.len() as u64);
    let mut prev = 0u32;
    for (i, &x) in sorted.iter().enumerate() {
        let d = if i == 0 { x } else { x - prev };
        put_varint(out, u64::from(d));
        prev = x;
    }
}

pub fn get_deltas(buf: &[u8], pos: &mut usize) -> Option<Vec<u32>> {
    let n = get_varint(buf, pos)?;
    let mut v = Vec::with_capacity(n as usize);
    let mut prev = 0u32;
    for i in 0..n {
        let d = u32::try_from(get_varint(buf, pos)?).ok()?;
        let x = if i == 0 { d } else { prev.checked_add(d)? };
        v.push(x);
        prev = x;
    }
    Some(v)
}

pub fn put_bytes(out: &mut Vec<u8>, b: &[u8]) {
    put_varint(out, b.len() as u64);
    out.extend_from_slice(b);
}

pub fn get_bytes<'a>(buf: &'a [u8], pos: &mut usize) -> Option<&'a [u8]> {
    let n = get_varint(buf, pos)? as usize;
    let s = buf.get(*pos..*pos + n)?;
    *pos += n;
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let ids = [0u32, 1, 1, 7, 300, 70_000, u32::MAX];
        let mut out = Vec::new();
        put_deltas(&mut out, &ids);
        let mut pos = 0;
        assert_eq!(get_deltas(&out, &mut pos).unwrap(), ids);
        assert_eq!(pos, out.len());
    }
}
