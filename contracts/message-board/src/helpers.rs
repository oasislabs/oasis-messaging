use oasis_std::prelude::*;
use serde_json::{from_slice, to_vec, Value};
use tiny_keccak::Keccak;

static MSG_KEY: &str = "message_key";
static FRIENDS_KEY: &str = "messaging_friends_key";

macro_rules! hash {
	($( $item:expr ),+) => {{
        let mut keccak = Keccak::new_keccak256();
        $( keccak.update($item); )+
        let mut res: [u8; 32] = [0; 32];
        keccak.finalize(&mut res);
        H256::from(res)
    }}
}

/// Convert a JSON object into a serialized byte array
pub fn to_bytes(json: Value) -> Vec<u8> {
    return to_vec(&json).expect("Could not serialize incoming JSON.");
}

/// Convert to JSON
pub fn from_bytes(bytes: Vec<u8>) -> Value {
    return from_slice(bytes.as_slice()).expect("Could not deserialize storable.");
}

/// Generate a unique key for storing a broadcast message using the message
/// count and owner of the message board
pub fn generate_broadcast_message_key(message_number: U256, owner: &Address) -> H256 {
    hash!(MSG_KEY.as_ref(), owner.as_ref(), {
        let mut bytes = vec![0; 32];
        message_number.to_little_endian(bytes.as_mut_slice());
        &bytes.clone()
    })
}

/// Generate a unique key for storing a directed message using a sender
/// and the recipient
/// Keccak hash generation is not commutative in that `hash(v1, v2) \ne hash(v2, v1)`
/// Since we want to retrieve a sequence of exchanges between two peers we
/// "canonicalize" the hash generation so that given a pair of peers, all of the
/// messages they exchanged are returned in sequence. Same applies to the
/// meta key generation function
pub fn generate_directed_message_key(
    message_number: U256,
    owner: &Address,
    sender: &Address,
    to: &Address,
) -> H256 {
    let mut addr1 = sender;
    let mut addr2 = to;
    if addr1.gt(addr2) {
        let t = addr1;
        addr1 = addr2;
        addr2 = t;
    }
    hash!(
        MSG_KEY.as_ref(),
        owner.as_ref(),
        addr1.as_ref(),
        addr2.as_ref(),
        {
            let mut bytes = vec![0; 32];
            message_number.to_little_endian(bytes.as_mut_slice());
            &bytes.clone()
        }
    )
}

/// Generate a unique key for storing a directed message using a sender
/// and the recipient
/// See comment under generate_directed_message_key for rationale to
/// conditionally swap the addresses
pub fn generate_directed_message_meta_key(owner: &Address, sender: &Address, to: &Address) -> H256 {
    let mut addr1 = sender;
    let mut addr2 = to;
    if addr1.gt(addr2) {
        let t = addr1;
        addr1 = addr2;
        addr2 = t;
    }
    hash!(
        MSG_KEY.as_ref(),
        owner.as_ref(),
        addr1.as_ref(),
        addr2.as_ref()
    )
}

/// Generate a unique key for storing the friends map
pub fn generate_friends_map_key(owner: &Address, addr: &Address) -> H256 {
    hash!(FRIENDS_KEY.as_ref(), owner.as_ref(), addr.as_ref())
}

/// Return the maximum number of messages that we want to return based
/// on number of messages on the board that haven't expired and the
/// desired number of messages requested
pub fn get_max_messages(k: u32, max: U256) -> u32 {
    if max > U256::from(u32::max_value()) {
        return k;
    }
    std::cmp::min(k, max.as_u32())
}

/// Check if the message is at most limit characters long
pub fn message_exceeds_limit(msg: &String, limit: usize) -> bool {
    if msg.len() > limit {
        let error_message = format!(
            "The message is longer than the {} character limit. Please shorten and re-send.",
            limit
        );
        log(&vec![], &error_message.into_bytes());
        true
    } else {
        false
    }
}
