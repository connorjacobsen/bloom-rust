use std::io::Cursor;
use std::mem;
use murmur3::murmur3_32;
use rand::Rng;

// The number of usize sized blocks in the bloom filter.
const M: usize = 1;

// The number of hash functions to use.
// Currently, the hash functions all use murmur3 with a randomly
// generated seed.
const K: usize = 3;

const BLOCK_SIZE: usize = mem::size_of::<u32>() * 8;

#[derive(Debug)]
pub struct Bloom {
    data: [u32; M],
    hash_seeds: [u32; K],
}

fn gen_seeds() -> [u32; K] {
    let mut rng  = rand::thread_rng();
    (0..K).map(|_| rng.gen()).collect::<Vec<u32>>().try_into().unwrap()
}

impl Bloom {
    fn new() -> Self {
        // Initialize all bits to 0.
        Bloom {
            data: [0; M],
            hash_seeds: gen_seeds(),
        }
    }

    fn hash_key(&self, key: &str) -> Result<[u32; K], String> {
        let mut hash_outputs: [u32; K] = [0; K];
        let mut cursor = Cursor::new(key);

        for (i, &seed) in self.hash_seeds.iter().enumerate() {
            match murmur3_32(&mut cursor, seed) {
                Ok(result) => hash_outputs[i] = result % ((BLOCK_SIZE * M) as u32),
                Err(reason) => return Err(reason.to_string())
            }
        }

        Ok(hash_outputs)
    }

    fn add(&mut self, key: &str) -> Result<(), String> {
        let hash_results = self.hash_key(key)?;

        for &result in hash_results.iter() {
            // TODO: come up with a cheaper way to determine the block.
            let block = (result / BLOCK_SIZE as u32) as usize;
            let block_pos = result % BLOCK_SIZE as u32;

            // Now we want to set the `block_pos` bit of the given `block`.
            self.data[block] |= 1 << block_pos
        }

        Ok(())
    }

    fn has_key(&self, key: &str) -> bool {
        let hash_results = self.hash_key(key).unwrap();

        for result in hash_results.iter() {
            let block = (result / BLOCK_SIZE as u32) as usize;
            let block_pos = result % (BLOCK_SIZE as u32);

            let bit_is_set = (self.data[block] & (1 << block_pos)) == (1 << block_pos);

            if !bit_is_set {
                return false;
            }
        }

        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_usage() {
        let mut bloom = Bloom::new();

        assert_eq!(bloom.add("key"), Ok(()));
        assert_eq!(bloom.has_key("key"), true);
        assert_eq!(bloom.has_key("def-not-a-key"), false)
    }
}
