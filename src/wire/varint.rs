use super::WireError;

/// Append `v` as an unsigned LEB128 varint.
#[allow(dead_code)] // encoder reference; the server only decodes today
pub fn write_varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let mut byte = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if v == 0 {
            break;
        }
    }
}

/// Read an unsigned LEB128 varint from the front of `input`, advancing it.
pub fn read_varint(input: &mut &[u8]) -> Result<u64, WireError> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        let byte = *input.first().ok_or(WireError::Truncated)?;
        *input = &input[1..];
        let payload = (byte & 0x7f) as u64;
        // Reject values that would overflow u64: a shift past bit 63, or a final
        // (10th) group carrying more than the single bit that fits at bit 63.
        if shift >= 64 || (shift == 63 && payload > 1) {
            return Err(WireError::Malformed);
        }
        result |= payload << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
    }
}

/// Split `n` bytes off the front of `input`, advancing it.
pub fn read_bytes<'a>(input: &mut &'a [u8], n: usize) -> Result<&'a [u8], WireError> {
    if input.len() < n {
        return Err(WireError::Truncated);
    }
    let (head, tail) = input.split_at(n);
    *input = tail;
    Ok(head)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_boundary_values() {
        for v in [0u64, 1, 127, 128, 16_383, 16_384, 1_000, u32::MAX as u64] {
            let mut buf = Vec::new();
            write_varint(&mut buf, v);
            let mut slice = &buf[..];
            assert_eq!(read_varint(&mut slice).unwrap(), v);
            assert!(slice.is_empty(), "consumed all bytes for {v}");
        }
    }

    #[test]
    fn small_values_are_one_byte() {
        let mut buf = Vec::new();
        write_varint(&mut buf, 127);
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn truncated_varint_errors() {
        let mut slice = &[0x80u8][..]; // continuation bit set, no follow-up
        assert!(matches!(read_varint(&mut slice), Err(WireError::Truncated)));
    }

    #[test]
    fn overflowing_varint_rejected() {
        // A 10th byte carrying payload > 1 overflows u64 (its bits shift past 63);
        // it must be rejected as Malformed, not silently wrapped. Here: nine 0x80
        // continuation bytes then 0x02 — currently wraps to 0.
        let bytes = [0x80u8, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x02];
        let mut slice = &bytes[..];
        assert!(matches!(read_varint(&mut slice), Err(WireError::Malformed)));
        // The maximal legal u64 (10 bytes, final 0x01) still decodes.
        let max = [0xffu8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01];
        let mut s = &max[..];
        assert_eq!(read_varint(&mut s).unwrap(), u64::MAX);
    }

    #[test]
    fn read_bytes_advances_and_bounds() {
        let data = [1u8, 2, 3, 4];
        let mut slice = &data[..];
        assert_eq!(read_bytes(&mut slice, 2).unwrap(), &[1, 2]);
        assert_eq!(slice, &[3, 4]);
        assert!(matches!(
            read_bytes(&mut slice, 3),
            Err(WireError::Truncated)
        ));
    }
}
