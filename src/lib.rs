pub mod bloomfilter;
pub mod heap;


mod utils {
    /// returns (n, mask), where the n-th byte has mask, all others 0
    pub(crate) fn get_single_bit_mask(idx: usize) -> (usize, u8) {
        let bits_elmt = std::mem::size_of::<u8>() * 8;

        let n = idx / bits_elmt; // the nth byte needs to get changed
        let idx_in_elmt = idx - (n * bits_elmt);
        assert!(idx_in_elmt < 8);
        let mask = 0x80u8 >> idx_in_elmt as u8;

        (n, mask)
    }

    /// sets all bits from bits_to_set indecies in the provided mask to 1
    pub(crate) fn set_multi_bitmask(mask: &mut [u8], bits_to_set: &mut dyn Iterator<Item = &usize>) {
        for idx in bits_to_set {
            let (n, b_mask) = get_single_bit_mask(*idx);
            assert!(n < mask.len(), "tried to flip byte outside of provided mask");
            mask[n] |= b_mask;
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn bit_mask_functions() {
        assert_eq!(crate::utils::get_single_bit_mask(0), (0, 0x80));
        assert_eq!(crate::utils::get_single_bit_mask(21), (2, 0x04));

        let mut mask = vec![0u8; 4];

        let bits = [2, 5, 11, 21, 22, 25, 31];

        crate::utils::set_multi_bitmask(&mut mask, &mut bits.iter());

        assert_eq!(&mask, &[0x24, 0x10, 0x06, 0x41]);
    }
}
