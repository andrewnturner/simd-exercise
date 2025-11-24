#[derive(Debug)]
pub enum DecodeError {
    InvalidByte(u8),
}

pub fn decode_base64_reference(data: &[u8], out: &mut Vec<u8>) -> Result<(), DecodeError> {
    // Strip off at most two b'=' from the end.
    let data = match data {
        [p @ .., b'=', b'='] => p,
        [p @ .., b'='] => p,
        p => p,
    };

    // Each ascii byte decodes to 6 bits, so 4 ascii bytes decodes to 3 bytes.
    for chunk in data.chunks(4) {
        // Buffer for at most 32 bits. We will decode 6 * chunk.len() < 32 bits.
        let mut decoded_bits = 0u32;

        for &ascii_byte in chunk {
            let sextet = match ascii_byte {
                b'A'..b'Z' => ascii_byte - b'A' + 0,
                b'a'..b'z' => ascii_byte - b'a' + 26,
                b'0'..b'9' => ascii_byte - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                _ => return Err(DecodeError::InvalidByte(ascii_byte)),
            };

            // Shift up our buffer by 6 bits and write the decoded sextet into the space.
            decoded_bits <<= 6;
            decoded_bits |= sextet as u32;
        }

        // Left align our decoded bits in the buffer.
        decoded_bits <<= 32 - (6 * chunk.len());

        let num_bytes_decoded = match chunk.len() {
            0 => unreachable!(),
            1 | 2 => 1,
            3 => 2,
            4 => 3,
            5.. => unreachable!(),
        };
        out.extend_from_slice(&decoded_bits.to_be_bytes()[..num_bytes_decoded]);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_base64_reference() {
        // "aaaa" -> 26 26 26 26 -> 011010 011010 011010 011010-> 01101001 10100110 10011010 -> 105 166 154
        let data = "aaaa";

        let mut out = Vec::new();
        decode_base64_reference(data.as_bytes(), &mut out).unwrap();

        let expected = vec![105, 166, 154];
        assert_eq!(out, expected);
    }
}
