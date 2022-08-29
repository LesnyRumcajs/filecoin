use std::env;

use cid::multihash::Code;
// use basic_token_actor::{MintParams, MintReturn};
use cid::Cid;
use factory_actor::{CreateFrc42Params, CreateFrc42Return};
use fil_fungible_token::token::state::TokenState;
use frc42_dispatch::method_hash;
use fvm::executor::{ApplyKind, Executor};
use fvm_integration_tests::tester::{Account, Tester};
use fvm_ipld_blockstore::{Block, MemoryBlockstore};
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::bigint::bigint_ser::BigIntDe;
use fvm_shared::bigint::Zero;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use fvm_shared::state::StateTreeVersion;
use fvm_shared::version::NetworkVersion;

const BASIC_TOKEN_ACTOR_WASM: &str =
    "../../target/debug/wbuild/basic_token_actor/basic_token_actor.compact.wasm";
const BASIC_RECEIVER_ACTOR_WASM: &str =
    "../../target/debug/wbuild/basic_receiving_actor/basic_receiving_actor.compact.wasm";
const FACTORY_ACTOR_WASM: &str =
    "../../target/debug/wbuild/factory_actor/factory_actor.compact.wasm";

#[test]
fn mint_tokens() {
    let blockstore = MemoryBlockstore::default();
    let mut tester =
        Tester::new(NetworkVersion::V15, StateTreeVersion::V4, blockstore.clone()).unwrap();

    let minter: [Account; 1] = tester.create_accounts().unwrap();

    let token_path =
        env::current_dir().unwrap().join(BASIC_TOKEN_ACTOR_WASM).canonicalize().unwrap();
    let token_bin = std::fs::read(token_path).expect("Unable to read token actor file");
    let rcvr_path =
        env::current_dir().unwrap().join(BASIC_RECEIVER_ACTOR_WASM).canonicalize().unwrap();
    let rcvr_bin = std::fs::read(rcvr_path).expect("Unable to read receiver actor file");

    let factory_path = env::current_dir().unwrap().join(FACTORY_ACTOR_WASM).canonicalize().unwrap();
    let factory_bin = std::fs::read(factory_path).expect("Unable to read factory actor file");

    // Set actor state
    let actor_state = TokenState::new(&blockstore).unwrap(); // TODO: this should probably not be exported from the package
    let state_cid = tester.set_state(&actor_state).unwrap();

    let token_address = Address::new_id(10000);
    let receive_address = Address::new_id(10001);
    let factory_address = Address::new_id(10002);
    let code_cid = tester
        .set_actor_from_bin(&token_bin, state_cid, token_address, TokenAmount::zero())
        .unwrap();
    tester
        .set_actor_from_bin(&rcvr_bin, Cid::default(), receive_address, TokenAmount::zero())
        .unwrap();
    tester
        .set_actor_from_bin(&factory_bin, Cid::default(), factory_address, TokenAmount::zero())
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

    // Use the factory to construct a token actor
    let ret_val = call_method(minter[0].1, factory_address, method_hash!("Constructor"), None);
    println!("factory actor constructor return data: {:#?}", &ret_val);

    // Construct the token actor
    let ret_val = call_method(minter[0].1, token_address, method_hash!("Constructor"), None);
    println!("token actor constructor return data: {:#?}", &ret_val);

    let ret_val = call_method(minter[0].1, receive_address, method_hash!("Constructor"), None);
    println!("receiving actor constructor return data: {:#?}", &ret_val);

    // Create the new actor
    let create_actor_params =
        CreateFrc42Params { code_cid, constructor_params: RawBytes::default() };
    let create_actor_params = RawBytes::serialize(create_actor_params).unwrap();
    let ret_val = call_method(
        minter[0].1,
        factory_address,
        method_hash!("CreateFrc42Token"),
        Some(create_actor_params),
    );
    println!("factory - CreateFrc42Params return {:#?}", &ret_val);
    let return_data = ret_val.msg_receipt.return_data;
    if return_data.is_empty() {
        println!("return data was empty");
    } else {
        let result: CreateFrc42Return = return_data.deserialize().unwrap();
        println!("result: {:#?}", &result);
    }

    // Check balance
    let params = RawBytes::serialize(receive_address).unwrap();
    let ret_val = call_method(minter[0].1, token_address, method_hash!("BalanceOf"), Some(params));
    println!("balance return data {:#?}", &ret_val);

    let return_data = ret_val.msg_receipt.return_data;
    let balance: BigIntDe = return_data.deserialize().unwrap();
    println!("balance: {:?}", balance.0);
}
