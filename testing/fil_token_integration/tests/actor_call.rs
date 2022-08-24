use std::env;

use cid::Cid;
use fil_fungible_token::receiver::types::TokenReceivedParams;
use fvm::executor::{ApplyKind, Executor};
use fvm_dispatch::method_hash;
use fvm_integration_tests::tester::{Account, Tester};
use fvm_ipld_blockstore::MemoryBlockstore;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::bigint::Zero;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use fvm_shared::state::StateTreeVersion;
use fvm_shared::version::NetworkVersion;

const BASIC_RECEIVER_ACTOR_WASM: &str =
    "../../target/debug/wbuild/basic_receiving_actor/basic_receiving_actor.compact.wasm";
const TEST_ACTOR_WASM: &str = "../../target/debug/wbuild/test_actor/test_actor.compact.wasm";

#[test]
fn actor_call() {
    let blockstore = MemoryBlockstore::default();
    let mut tester =
        Tester::new(NetworkVersion::V15, StateTreeVersion::V4, blockstore.clone()).unwrap();

    let minter: [Account; 1] = tester.create_accounts().unwrap();

    // Get wasm bin
    let test_path = env::current_dir().unwrap().join(TEST_ACTOR_WASM).canonicalize().unwrap();
    let test_bin = std::fs::read(test_path).expect("Unable to read test actor file");
    let rcvr_path =
        env::current_dir().unwrap().join(BASIC_RECEIVER_ACTOR_WASM).canonicalize().unwrap();
    let rcvr_bin = std::fs::read(rcvr_path).expect("Unable to read receiver actor file");

    let actor_address = Address::new_id(10000);
    let receive_address = Address::new_id(10010);
    tester
        .set_actor_from_bin(&test_bin, Cid::default(), actor_address, TokenAmount::zero())
        .unwrap();
    tester
        .set_actor_from_bin(&rcvr_bin, Cid::default(), receive_address, TokenAmount::zero())
        .unwrap();

    // Instantiate machine
    tester.instantiate_machine().unwrap();

    // Helper to simplify sending messages
    let mut sequence = 0u64;
    let mut call_method = |from, to, method_num, params| {
        let message = Message {
            from,
            to,
            gas_limit: 99999999,
            method_num,
            sequence,
            params: if let Some(params) = params { params } else { RawBytes::default() },
            ..Message::default()
        };
        sequence += 1;
        tester
            .executor
            .as_mut()
            .unwrap()
            .execute_message(message, ApplyKind::Explicit, 100)
            .unwrap()
    };

    // Construct the token actor
    let ret_val = call_method(minter[0].1, actor_address, method_hash!("Constructor"), None);
    println!("test actor constructor return data: {:?}", &ret_val);
    let ret_val = call_method(minter[0].1, receive_address, method_hash!("Constructor"), None);
    println!("receiving actor constructor return data: {:?}", &ret_val);

    // Call test actor with the receiver hook actor's address
    // each method called will generate some test data and call methods on the supplied address
    let params = RawBytes::serialize(&receive_address).unwrap();
    let ret_val =
        call_method(minter[0].1, actor_address, method_hash!("TokensReceived"), Some(params));
    println!("\nreceive hook return data {:?}", &ret_val);
    println!("{:?}", ret_val.msg_receipt.return_data.deserialize::<String>());

    let params = RawBytes::serialize(&receive_address).unwrap();
    let ret_val = call_method(minter[0].1, actor_address, method_hash!("TakeString"), Some(params));
    println!("\ntake-string return data {:?}", &ret_val);
    println!("{:?}", ret_val.msg_receipt.return_data.deserialize::<String>());

    let params = RawBytes::serialize(&receive_address).unwrap();
    let ret_val =
        call_method(minter[0].1, actor_address, method_hash!("TakeInteger"), Some(params));
    println!("\ntake-integer return data {:?}", &ret_val);
    println!("{:?}", ret_val.msg_receipt.return_data.deserialize::<String>());

    let params = RawBytes::serialize(&receive_address).unwrap();
    let ret_val = call_method(minter[0].1, actor_address, method_hash!("TakeStruct"), Some(params));
    println!("\ntake-struct return data {:?}", &ret_val);
    println!("{:?}", ret_val.msg_receipt.return_data.deserialize::<String>());

    println!("blockstore contents: {:?}", blockstore);
}
