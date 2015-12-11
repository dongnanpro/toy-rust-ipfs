mod pb;

use blockstore::Blockstore;
use util;

use protobuf::{self, Message, RepeatedField};
use rust_multihash::Multihash;
use std::io::{Read, Write};
use std::sync::RwLock;

// TODO: pointer to node. How to do?
struct Link {
    name: String,
    hash: Multihash,
    target_size: u64,
}

struct Node {
    data: Vec<u8>,
    links: Vec<Link>,
    multihash: RwLock<Option<Multihash>>, // caches the multihash so it isn't recomputed
}

pub struct DagService {
    blockstore: Blockstore,
}

impl Link {
    pub fn from_pblink(mut link: pb::PBLink) -> Self {
        Link {
            name: link.take_Name(),
            hash: util::hash(link.get_Hash()),
            target_size: link.get_Tsize(),
        }
    }

    pub fn clone_to_pblink(&self) -> pb::PBLink {
        let mut pblink = pb::PBLink::new();
        pblink.set_Name(self.name.clone());
        pblink.set_Hash(self.hash.clone().into_bytes());
        pblink.set_Tsize(self.target_size);
        pblink
    }
}

impl Node {
    pub fn get_data(&self) -> &[u8] { &self.data[..] }

    pub fn multihash(&self) -> Multihash {
        match self.multihash.try_read() {
            Ok(ref cache) if cache.is_some() => cache.as_ref().unwrap().clone(),
            _ => {
                let mh = self.calculate_multihash();

                let cache_lock = self.multihash.try_write();
                if cache_lock.is_ok() {
                    *cache_lock.unwrap() = Some(mh.clone());
                }
                mh
            },
        }
    }

    fn calculate_multihash(&self) -> Multihash {
        let mut buf = Vec::new();
        self.encode_to_writer(&mut buf);
        util::hash(&buf[..])
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, String> {
        let mut pbnode = try!(protobuf::parse_from_reader::<pb::PBNode>(reader)
                              .map_err(|e| format!("Error parsing encoded Node: {}", e)));

        let mut links = Vec::new();

        for link in pbnode.take_Links().into_iter() {
            links.push(Link::from_pblink(link));
        }

        Ok(Node {
            data: pbnode.take_Data(),
            links: links,
            multihash: RwLock::new(None),
        })
    }

    pub fn encode_to_writer<W: Write>(&self, writer: &mut W) -> Result<(), String> {
        let mut pbnode = pb::PBNode::new();
        pbnode.set_Data(self.data.clone());

        // TODO: go-ipfs sorts the links by name before hashing
        let pblinks = self.links.iter().map(|link| link.clone_to_pblink()).collect();
        pbnode.set_Links(RepeatedField::from_vec(pblinks));

        pbnode.write_to_writer(writer)
              .map_err(|e| format!("Error cloning node to writer: {}", e))
    }
}
