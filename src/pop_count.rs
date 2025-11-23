pub fn pop_count_reference(x: u32) -> u32 {
    let mut count = 0;
    for i in 0..32 {
        let has_bit_i = (x >> i) & 1;
        count += has_bit_i;
    }

    count
}

pub fn pop_count(mut x: u32) -> u32 {
    // The pop count is the sum of the values when we interpret as 32 one bit
    // integers.

    // Split into the even and odd indexed bits, then shift the odd ones down to
    // align with the even ones. When we add, each two bit pair can't overflow
    // since the values in each are either 0b00 or 0b01.
    // The pop count is now the sum of the values when we interpret as 16 two
    // bit integers.
    let even = x & 0x55555555; // 0x5 == 0b0101
    let odd = x & 0xaaaaaaaa; // 0xa == 0b1010
    x = even + (odd >> 1);

    // Now split into even and odd two bit pairs, and shift the odd ones down
    // two places to align. Again each four bit group can't overflow when we add.
    // The pop count is now the sum of the values when we interpret as 8 four
    // bit integers.
    let even = x & 0x33333333; // 0x3 == 0b0011;
    let odd = x & 0xcccccccc; // 0xc == 0b1100;
    x = even + (odd >> 2);

    // Continue the pattern.
    let even = x & 0x0f0f0f0f;
    let odd = x & 0xf0f0f0f0;
    x = even + (odd >> 4);

    let even = x & 0x00ff00ff;
    let odd = x & 0xff00ff00;
    x = even + (odd >> 8);

    let even = x & 0x0000ffff;
    let odd = x & 0xffff0000;
    x = even + (odd >> 16);

    // The pop count is now just the value interpreted as a single u32.
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop_count_reference() {
        // 37 = 32 + 4 + 1
        assert_eq!(pop_count_reference(37), 3);
    }

    #[test]
    fn test_pop_count() {
        for i in 0..252 {
            assert_eq!(pop_count(i), pop_count_reference(i));
        }
    }
}
