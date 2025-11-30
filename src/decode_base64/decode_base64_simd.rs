use std::simd::cmp::SimdPartialEq;
use std::simd::num::{SimdInt, SimdUint};
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

#[derive(Debug)]
pub enum DecodeError {
    InvalidByte,
}

pub fn decode_base64_simd(data: &[u8], out: &mut Vec<u8>) -> Result<(), DecodeError> {
    // Strip off at most two b'=' from the end.
    let data = match data {
        [p @ .., b'=', b'='] => p,
        [p @ .., b'='] => p,
        p => p,
    };

    // Each ascii byte decodes to 6 bits, so 4 ascii bytes decodes to 3 bytes.
    for chunk in data.chunks(4) {
        // Padding with b'A' won't change the value since it maps to 0;
        let mut ascii = [b'A'; 8];
        ascii[..chunk.len()].copy_from_slice(chunk);

        let (decoded_bytes, is_ok) = decode_hot(ascii.into());
        if !is_ok {
            return Err(DecodeError::InvalidByte);
        }

        // chunk_len:                                      1 2 3 4
        // chunk_len / 2:                                  0 1 1 2
        // chunk_len - (chunk_len / 2):                    1 1 2 2
        // chunk_len / 4:                                  0 0 0 1
        // chunk_len - (chunk_len / 2) + (chunk_len / 4):  1 1 2 3
        let chunk_len = chunk.len();
        let num_bytes_decoded = chunk_len - (chunk_len / 2) + (chunk_len / 4);

        out.extend_from_slice(&decoded_bytes[..num_bytes_decoded]);
    }

    Ok(())
}

const fn tiled<T, const N: usize>(tile: &[T]) -> Simd<T, N>
where
    T: SimdElement,
    LaneCount<N>: SupportedLaneCount,
{
    let mut out = [tile[0]; N];

    // Can't use range in const context.
    let mut i = 0;
    while i < N {
        out[i] = tile[i % tile.len()];
        i += 1;
    }

    Simd::from_array(out)
}

fn build_selections<const N: usize>() -> Simd<u8, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    let mut out = [0; N];

    let mut i = 0;
    while i < N {
        out[i] = (i + (i / 3)) as u8;
        i += 1;
    }

    Simd::from_array(out)
}

fn decode_hot<const N: usize>(ascii: Simd<u8, N>) -> (Simd<u8, N>, bool)
where
    LaneCount<N>: SupportedLaneCount,
{
    // This is enough to differentiate the ranges:
    //   - A..Z == 0x41..0x5b => 4, 5
    //   - a..z == 0x61..0x7b => 6, 7
    //   - 0..9 == 0x30..0x3a => 3
    //   - +    == 0x2b       => 2
    //   - /    == 0x2f       => 1
    let hashes = (ascii >> Simd::splat(4))
        + Simd::simd_eq(ascii, Simd::splat(b'/'))
            .to_int()
            .cast::<u8>();

    // Maps the hash to the corresponding offset:
    //   - A..Z => - b'A' + 0  = -65
    //   - a..z => - b'a' + 26 = -71
    //   - 0..9 => - b'0' + 52 = 4
    //   - +    => - b'+' + 62 = 19
    //   - /    =? - b'/' + 63 = 16
    // Value for hash 0 is unused.
    let offsets_table: [i8; 8] = [0, 16, 19, 4, -65, -65, -71, -71];
    let tiled_offsets_table = tiled(&offsets_table);

    // Have to use u8 for swizzle_dyn.
    // With two's complement, positives go to same value while negatives go to 256 + the value.
    let offsets = tiled_offsets_table.cast::<u8>().swizzle_dyn(hashes);

    // Positive offsets are applied as normal, negative offsets which be 256 too big which overflows
    // off giving us the correct value.
    let sextets = ascii + offsets;

    let is_ok = true; // TODO

    // Casting to u16 adds zeros to the left. Then we shift each lane by the correct amount so the
    // chunks we want to manipulate are at the 8-bit boundaries, and finally can extract the low and
    // high halves from each. Then we combine the low part with the next lane's high part to pack.
    //
    // sextets:   | aaa. | bbb. | ccc. | ddd. |
    // low_bits:  | .aaa | ..bb | ...c | .... |
    // high_bits: | .... | b... | cc.. | ddd. |
    // rotated:   | b... | cc.. | ddd. | .... |
    // packed:    | baaa | ccbb | dddc | .... |
    let shifted = sextets.cast::<u16>() << tiled(&[2, 4, 6, 8]);
    let low_bits = shifted.cast::<u8>();
    let high_bits = (shifted >> Simd::splat(8)).cast::<u8>();

    let packed = low_bits | high_bits.rotate_elements_left::<1>();

    // Drop every fourth lane.
    // TODO: ideally would use`simd_swizzle` and `selections` woud be const. But the macro doesn't
    // seem to work with calls to a const function currently.
    let selections = build_selections();
    let output = packed.swizzle_dyn(selections);

    (output, is_ok)
}

#[cfg(test)]
mod tests {
    use crate::decode_base64::decode_base64_reference;

    use super::*;

    #[test]
    fn test_decode_base64_simd() {
        // "aaaa" -> 26 26 26 26 -> 011010 011010 011010 011010-> 01101001 10100110 10011010 -> 105 166 154
        let data = "aaaaaaaa";

        let mut out = Vec::new();
        decode_base64_simd(data.as_bytes(), &mut out).unwrap();

        let expected = vec![105, 166, 154, 105, 166, 154];
        assert_eq!(out, expected);
    }

    #[test]
    fn test_decode_base64_simd_first_char() {
        let data = "a";

        let mut out = Vec::new();
        decode_base64_simd(data.as_bytes(), &mut out).unwrap();

        let expected = vec![104];
        assert_eq!(out, expected);
    }

    #[test]
    fn test_decode_base64_simd_second_char() {
        let data = "AaA";

        let mut out = Vec::new();
        decode_base64_simd(data.as_bytes(), &mut out).unwrap();

        let expected = vec![1, 160];
        assert_eq!(out, expected);
    }

    #[test]
    fn test_decode_base64_simd_against_reference() {
        let data = "jhgsdf6234hsdf";

        let mut out = Vec::new();
        decode_base64_simd(data.as_bytes(), &mut out).unwrap();

        let mut out_reference = Vec::new();
        decode_base64_reference(data.as_bytes(), &mut out_reference).unwrap();

        assert_eq!(out, out_reference);
    }
}
