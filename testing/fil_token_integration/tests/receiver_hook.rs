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

#[test]
fn receiver_hook() {
    let blockstore = MemoryBlockstore::default();
    let mut tester =
        Tester::new(NetworkVersion::V15, StateTreeVersion::V4, blockstore.clone()).unwrap();

    let minter: [Account; 1] = tester.create_accounts().unwrap();

    // Get wasm bin
    let rcvr_path =
        env::current_dir().unwrap().join(BASIC_RECEIVER_ACTOR_WASM).canonicalize().unwrap();
    let rcvr_bin = std::fs::read(rcvr_path).expect("Unable to read receiver actor file");

    let actor_address = Address::new_id(10000);
    let receive_address = Address::new_id(10010);
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
    let ret_val = call_method(minter[0].1, receive_address, method_hash!("Constructor"), None);
    println!("receiving actor constructor return data: {:#?}", &ret_val);

    // Call receiver hook with some fake minting data
    let recv_params = TokenReceivedParams {
        operator: 10000u64,
        from: 10000u64,
        to: 10010u64,
        amount: TokenAmount::from(100),
        data: Default::default(),
    };
    let params = RawBytes::serialize(recv_params).unwrap();
    let ret_val =
        call_method(minter[0].1, receive_address, method_hash!("TokensReceived"), Some(params));
    println!("mint return data {:#?}", &ret_val);
    //let return_data = ret_val.msg_receipt.return_data;

    println!("blockstore contents: {:?}", blockstore);
}
