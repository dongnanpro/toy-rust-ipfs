use super::MultiHash;
use super::datastore::DSKey;
use super::util;

// A singular block of data in IPFS
pub struct Block {
    mh: MultiHash,
    data: Vec<u8>,
}

// Some constructors for `Block`, as well as the `key` method.
impl Block {
    pub fn new() -> Block {
        Block {
            mh: MultiHash::new(),
            data: Vec::new(),
        }
    }

    pub fn data(data: Vec<u8>) -> Block {
        Block {
            mh: util::hash(&data[..]),
            data: data,
        }
    }

    // This function trusts that the MultiHash `mh` is
    // correct for `data`
    //
    //  TODO: Have a debug mode where we go ahead and
    //  encode `data` to verify that `mh` is correct
    pub fn data_with_hash(data: Vec<u8>, mh: MultiHash) -> Block {
        Block {
            mh: mh,
            data: data,
        }
    }


    pub fn key(&self) -> DSKey {
        DSKey::new_key(self.mh.to_base58_string())
    }
}
