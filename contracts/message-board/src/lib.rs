extern crate hex;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate tiny_keccak;

pub mod helpers;

use std::collections::HashSet;

use oasis_std::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, to_value};

use helpers::*;

static CHAR_LIMIT_KEY: [u8; 32] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static CREATOR_KEY: [u8; 32] = [
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static MESSAGE_NUM_KEY: [u8; 32] = [
    3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static MESSAGE_LENGTH_LIMIT: usize = 1024;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct BroadcastMessage {
    sender: Address,
    message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DirectedMessage {
    sender: Address,
    recipient: Address,
    message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct FriendsMap {
    friends: HashSet<String>,
}

macro_rules! read_messages {
    ($message_type:ty, $getter:ident, $message_num:ident, $k:ident, $( $getter_arg:expr ),+) => {{
        let max_messages = get_max_messages($k, $message_num);
        let mut result = json!({});
        for index in 0..max_messages {
            let message_index = $message_num - index - 1;
            let message_key = $getter(message_index, $( $getter_arg, )+);
            let message_bytes = get_bytes(&message_key);
            let message: $message_type =
                from_value(helpers::from_bytes(message_bytes.unwrap())).unwrap();
            let index_str = format!("{}", message_index);
            result[index_str] = to_value(message).unwrap();
        }

        // return stringified byteified result
        let result_str = format!("{}", &result);
        result_str.into_bytes()
    }}
}

macro_rules! read_message_by_index {
    ($message_type:ty, $getter:ident, $message_num:ident, $k:ident, $( $getter_arg:expr ),+) => {{
        if $message_num > U256::from(u32::max_value()) || $k <= $message_num.as_u32() {
            let message_index = $message_num - $k - 1;
            let message_key = $getter(message_index, $( $getter_arg, )+);
            let message_bytes = get_bytes(&message_key);
            let message: $message_type =
                from_value(helpers::from_bytes(message_bytes.unwrap())).unwrap();
            let result = to_value(message).unwrap();
            result["message"].to_string()
        } else {
            "".to_string()
        }
    }}
}

/// Update the friends map
/// For a given person, updates the friends list with a friend
/// Note: For efficiency we don't store and retrieve a hash map
/// but instead store a hash set of friends for each person
fn update_friend(owner: &Address, me: &Address, friend: &Address) {
    let key = generate_friends_map_key(owner, me);
    let bytes = get_bytes(&key).unwrap();
    let mut friends_map = FriendsMap::default();
    if !bytes.is_empty() {
        friends_map = from_value(helpers::from_bytes(bytes)).unwrap();
    }
    friends_map.friends.insert(hex::encode(friend));
    let json_bytes = helpers::to_bytes(to_value(friends_map).unwrap());
    if let Err(_) = set_bytes(&key, &json_bytes) {
        log(
            &vec![],
            &"Failed to update friends map".to_string().into_bytes(),
        )
    }
}

#[contract]
trait MessageBoard {
    fn constructor(&mut self, char_limit: U256) {
        if char_limit.as_usize() > MESSAGE_LENGTH_LIMIT {
            let error_message = format!(
                "The maximum allowed char limit for messages is {}. PLease pick a number <= {}.",
                MESSAGE_LENGTH_LIMIT, MESSAGE_LENGTH_LIMIT
            );
            log(&vec![], &error_message.into_bytes());
            return;
        }
        let sender = sender();

        // Stash the char limit
        write(&CHAR_LIMIT_KEY.into(), &char_limit.into());

        // Set the contract owner
        write(&CREATOR_KEY.into(), &H256::from(sender).into());

        let message_num: U256 = U256::zero();
        write(&MESSAGE_NUM_KEY.into(), &message_num.into());
    }

    #[constant]
    fn get_char_limit(&mut self) -> U256 {
        read(&CHAR_LIMIT_KEY.into()).into()
    }

    /// Function to post a message on the message board
    fn post(&mut self, message: String) -> bool {
        let mut message_num: U256 = read(&MESSAGE_NUM_KEY.into()).into();
        let char_limit: U256 = read(&CHAR_LIMIT_KEY.into()).into();
        let sender = sender();
        if message_exceeds_limit(&message, char_limit.as_usize()) {
            return false;
        }

        let owner = Address::from(H256::from(read(&CREATOR_KEY.into())));
        let broadcast_message = BroadcastMessage { sender, message };
        let json_bytes = helpers::to_bytes(to_value(broadcast_message).unwrap());
        let broadcast_message_key = generate_broadcast_message_key(message_num, &owner);
        if let Err(_) = set_bytes(&broadcast_message_key, &json_bytes) {
            log(
                &vec![],
                &"Failed to store broadcast message".to_string().into_bytes(),
            )
        }

        // increment message_num and store
        message_num = message_num + 1;
        write(&MESSAGE_NUM_KEY.into(), &message_num.into());

        true
    }

    /// Get all non-expired broadcast messages up to k messages
    #[constant]
    fn get_broadcast_messages(&mut self, k: u32) -> Vec<u8> {
        let owner = Address::from(H256::from(read(&CREATOR_KEY.into())));
        let message_num: U256 = read(&MESSAGE_NUM_KEY.into()).into();
        read_messages!(
            BroadcastMessage,
            generate_broadcast_message_key,
            message_num,
            k,
            &owner
        )
    }

    /// Get a non-expired broadcast message by index
    #[constant]
    fn get_broadcast_message_by_index(&mut self, k: u32) -> String {
        let owner = Address::from(H256::from(read(&CREATOR_KEY.into())));
        let message_num: U256 = read(&MESSAGE_NUM_KEY.into()).into();
        read_message_by_index!(
            BroadcastMessage,
            generate_broadcast_message_key,
            message_num,
            k,
            &owner
        )
    }

    /// Function to send a message to a specific recipient
    fn send(&mut self, to: Address, message: String) -> bool {
        let sender = sender();
        let owner = Address::from(H256::from(read(&CREATOR_KEY.into())));
        let directed_message_meta_key = generate_directed_message_meta_key(&owner, &sender, &to);
        let mut message_num: U256 = read(&directed_message_meta_key.into()).into();
        let char_limit: U256 = read(&CHAR_LIMIT_KEY.into()).into();
        if message_exceeds_limit(&message, char_limit.as_usize()) {
            return false;
        }

        let directed_message = DirectedMessage {
            sender: sender,
            recipient: to,
            message: message,
        };
        let json_bytes = helpers::to_bytes(to_value(directed_message).unwrap());
        let directed_message_key = generate_directed_message_key(message_num, &owner, &sender, &to);
        if let Err(_) = set_bytes(&directed_message_key, &json_bytes) {
            log(
                &vec![],
                &"Failed to store directed message".to_string().into_bytes(),
            )
        }

        // increment message_num and store
        message_num = message_num + 1;
        write(&directed_message_meta_key.into(), &message_num.into());

        update_friend(&owner, &sender, &to);
        update_friend(&owner, &to, &sender);

        true
    }

    /// Get friends
    #[constant]
    fn get_friends(&mut self, sender: Address) -> Vec<u8> {
        let owner = Address::from(H256::from(read(&CREATOR_KEY.into())));
        let key = generate_friends_map_key(&owner, &sender);
        let bytes = get_bytes(&key).unwrap();
        if !bytes.is_empty() {
            let friends_map: FriendsMap = from_value(helpers::from_bytes(bytes)).unwrap();
            let result_str = format!("{}", to_value(friends_map).unwrap());
            result_str.into_bytes()
        } else {
            Vec::new()
        }
    }

    #[constant]
    fn get_friends_as_string(&mut self, sender: Address) -> String {
        let owner = Address::from(H256::from(read(&CREATOR_KEY.into())));
        let key = generate_friends_map_key(&owner, &sender);
        let bytes = get_bytes(&key).unwrap();
        if !bytes.is_empty() {
            let friends_map: FriendsMap = from_value(helpers::from_bytes(bytes)).unwrap();
            friends_map
                .friends
                .iter()
                .fold(String::new(), |a, b| a + " " + &b)
        } else {
            "".to_string()
        }
    }

    /// Get the last k messages, in sequence, that were exchanged between a sender and a recipient
    #[constant]
    fn get_messages(&mut self, sender: Address, to: Address, k: u32) -> Vec<u8> {
        let owner = Address::from(H256::from(read(&CREATOR_KEY.into())));
        let directed_message_meta_key = generate_directed_message_meta_key(&owner, &sender, &to);
        let message_num: U256 = read(&directed_message_meta_key.into()).into();
        read_messages!(
            DirectedMessage,
            generate_directed_message_key,
            message_num,
            k,
            &owner,
            &sender,
            &to
        )
    }

    /// Get a message by index for a given sender, recipient pair
    #[constant]
    fn get_message_by_index(&mut self, sender: Address, to: Address, k: u32) -> String {
        let owner = Address::from(H256::from(read(&CREATOR_KEY.into())));
        let directed_message_meta_key = generate_directed_message_meta_key(&owner, &sender, &to);
        let message_num: U256 = read(&directed_message_meta_key.into()).into();
        read_message_by_index!(
            DirectedMessage,
            generate_directed_message_key,
            message_num,
            k,
            &owner,
            &sender,
            &to
        )
    }
}
