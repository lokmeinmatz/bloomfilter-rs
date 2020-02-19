use std::hash::{Hash, Hasher, BuildHasher};
use std::fmt::{Binary, Formatter, Error};
use std::collections::hash_map::{DefaultHasher, RandomState};

pub struct BloomFilter<Storage = Vec<u8>, H = DefaultHasher> {
    data: Storage,
    data_len: usize,
    elmts_added: usize,
    hashers: Vec<H>
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


    fn get_bit_indecies<E: Hash>(&self, elmt: & E) -> Vec<usize> {

        let store_len = self.data_len * 8;

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

        let store = self.data.as_ref();



        for (n, mask) in
        self.get_bit_indecies(elmt).iter().map(|i| crate::utils::get_single_bit_mask(*i)) {
            if store[n] & mask != mask { return true; }
        }

        false
    }


    /// Add a hashable Element to the BloomFilter.
    /// print
    pub fn add<E: Hash>(&mut self, elmt: &E) {
        self.elmts_added += 1;

        for (n, mask) in
        self.get_bit_indecies(elmt).iter().map(|i| crate::utils::get_single_bit_mask(*i)) {
            self.data.as_mut()[n] |= mask;
        }
    }

    pub fn err_probability(&self) -> f64 {
        let fill_ratio = self.data.as_ref().iter().map(|e| e.count_ones()).sum::<u32>() as f64 / (self.data_len * 8) as f64;

        fill_ratio.powi(self.hashers.len() as i32)
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
    use crate::bloomfilter::BloomFilter;

    #[test]
    fn filter_test_basic() {

        let mut filter = BloomFilter::default_with_settings(16, 4);

        // filter shouldn't fire any value
        assert!((0..100).all(|e| filter.never_occured(&e)));

        println!("{:b}", filter);
        println!("ErrProb.: {}", filter.err_probability());
        filter.add(&2);
        filter.add(&4);

        println!("{:b}", filter);
        println!("ErrProb.: {}", filter.err_probability());
        assert!(!filter.never_occured(&2));
        assert!(!filter.never_occured(&4));

        // this may fail if h(23) & (h(2) | h(4)) != 0 where h is the multi-bitmask
        assert!(filter.never_occured(&3334));

        for i in 10..20 {
            filter.add(&i);
        }

        println!("{:b}", filter);
        println!("ErrProb.: {}", filter.err_probability());
    }
}
