pub fn pop_count_reference(x: u32) -> u32 {
    let mut count = 0;
    for i in 0..32 {
        let has_bit_i = (x >> i) & 1;
        count += has_bit_i;
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop_count_reference() {
        // 37 = 32 + 4 + 1
        assert_eq!(pop_count_reference(37), 3);
    }
}
