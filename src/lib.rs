use std::hash::{Hash, Hasher, BuildHasher};
use std::collections::hash_map::{DefaultHasher, RandomState};
use std::fmt::{Formatter, Error, Binary};

pub struct BloomFilter<Storage = Vec<u8>, H = DefaultHasher> {
    data: Storage,
    data_len: usize,
    elmts_added: usize,
    hashers: Vec<H>
}


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

impl BloomFilter<Vec<u8>, DefaultHasher> {
    /// creates new BloomFilter with `n_bytes` bytes of storage (n * 8 bits)
    /// and an default storage of type [`Vec<u8>`](std::vec::Vec) and `m_hashers` different Hasher,
    /// so each bit gets a different position in the storage when [`add`](#add) is called.
    pub fn default_with_settings(n_bytes: usize, m_hashers: usize) -> Self {
        let store = vec![0u8; n_bytes];

        let hashers: Vec<DefaultHasher> = (0..m_hashers).map(|_| {
            RandomState::new().build_hasher()
        }).collect();

        BloomFilter::from_initalized(store, hashers)
    }
}

impl <Storage: AsRef<[u8]> + AsMut<[u8]>, H: Hasher + Clone> BloomFilter<Storage, H> {
    pub fn from_initalized(store: Storage, hashers: Vec<H>) -> Self {
        let len = store.as_ref().len();
        BloomFilter {
            data: store,
            data_len: len,
            elmts_added: 0,
            hashers
        }
    }


    /// How many bytes the storage for the bitmask uses.
    ///
    /// The Filter can store `storage_size * 8` bits.
    pub fn storage_size(&self) -> usize {
        self.data_len
    }

    /// How many hashes get calculated for each call to [`add`](#add) or [`never_occured`](#never_occured).
    pub fn num_hashers(&self) -> usize {
        self.hashers.len()
    }


    fn get_bit_indecies<'a, E: Hash>(&'a self, elmt: &'a E) -> Vec<usize> {
        let store_slice: &[u8] = self.data.as_ref();

        let store_len = store_slice.len() * 8;

        self.hashers.iter().map(|h| {
            let mut h: H = (*h).clone();
            (&*elmt).hash(&mut h);
            h.finish() as usize % store_len
        }).collect()
    }


    /// If this function returns true, the value was NEVER added to this [`BloomFilter`].
    ///
    /// **make sure that this only applies to values that produce the same hash**, eg if the
    /// hash of the object changes, we don't know any more if it was added :(
    pub fn never_occured<E: Hash>(&self, elmt: &E) -> bool {

        let mut el_mask = vec![0u8; self.data_len];

        crate::utils::set_multi_bitmask(&mut el_mask, &mut self.get_bit_indecies(elmt).iter());

        for (b_mask_el, b_mask_store) in el_mask.iter().zip(self.data.as_ref().iter()) {
            if *b_mask_el & *b_mask_store != *b_mask_el { return true; }
        }

        false
    }


    /// Add a hashable Element to the BloomFilter.
    /// print
    pub fn add<E: Hash>(&mut self, elmt: &E) {

        let mut el_mask = vec![0u8; self.data_len];

        crate::utils::set_multi_bitmask(&mut el_mask, &mut self.get_bit_indecies(elmt).iter());

        for (b_mask_el, b_mask_store) in el_mask.iter().zip(self.data.as_mut().iter_mut()) {
            *b_mask_store |= *b_mask_el;
        }
    }
}

#[cfg(feature = "debug")]
impl Binary for BloomFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut bits = String::with_capacity(self.data_len * 8);
        for byte in &self.data {
            let mut byte = *byte;
            for _ in 0..8 {
                bits.push(if byte & 0x80 == 0 {'0'} else {'1'});
                byte = byte << 1;
            }
        }
        write!(f, "BloomFilter binary {{ {} }}", bits)
    }
}

#[cfg(test)]
mod tests {
    use crate::BloomFilter;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn bit_mask_functions() {
        assert_eq!(crate::utils::get_single_bit_mask(0), (0, 0x80));
        assert_eq!(crate::utils::get_single_bit_mask(21), (2, 0x04));

        let mut mask = vec![0u8; 4];

        let bits = [2, 5, 11, 21, 22, 25, 31];

        crate::utils::set_multi_bitmask(&mut mask, &mut bits.iter());

        assert_eq!(&mask, &[0x24, 0x10, 0x06, 0x41]);
    }

    #[test]
    fn filter_test_basic() {

        let mut filter = BloomFilter::default_with_settings(4, 4);

        // filter shouldn't fire any value
        assert!((0..100).all(|e| filter.never_occured(&e)));

        println!("{:b}", filter);

        filter.add(&2);
        filter.add(&4);

        println!("{:b}", filter);
        assert!(!filter.never_occured(&2));
        assert!(!filter.never_occured(&4));

        // this may fail if h(23) & (h(2) | h(4)) != 0 where h is the multi-bitmask
        assert!(filter.never_occured(&3334));
    }
}
