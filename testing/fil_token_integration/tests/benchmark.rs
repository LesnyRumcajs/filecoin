use std::env;

use fil_fungible_token::token::state::TokenState;
use frc42_dispatch::method_hash;
use fvm::executor::{ApplyKind, Executor};
use fvm_integration_tests::bundle;
use fvm_integration_tests::dummy::DummyExterns;
use fvm_integration_tests::tester::{Account, Tester};
use fvm_ipld_blockstore::MemoryBlockstore;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::bigint::Zero;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use fvm_shared::state::StateTreeVersion;
use fvm_shared::version::NetworkVersion;

const BENCHMARK_ACTOR_WASM: &str =
    "../../target/debug/wbuild/benchmark_actor/benchmark_actor.compact.wasm";

#[test]
fn benchmark_hamt() {
    let test_vectors = vec![
        (0x1, 0x1),
        (0x1, 0x2),
        (0x1, 0x3),
        (0x2, 0x1),
        (0x2, 0x2),
        (0x2, 0x3),
        (0x3, 0x1),
        (0x3, 0x2),
        (0x3, 0x3),
        (0x4, 0x1),
        (0x4, 0x2),
        (0x4, 0x3),
    ];

    for test_case in test_vectors {
        let blockstore = MemoryBlockstore::default();
        let bundle_root = bundle::import_bundle(&blockstore, actors_v10::BUNDLE_CAR).unwrap();
        let mut tester =
            Tester::new(NetworkVersion::V15, StateTreeVersion::V4, bundle_root, blockstore.clone())
                .unwrap();

        let actor_address = Address::new_id(u64::MAX);

        let actor: [Account; 1] = tester.create_accounts().unwrap();

        // Get wasm bin
        let wasm_path =
            env::current_dir().unwrap().join(BENCHMARK_ACTOR_WASM).canonicalize().unwrap();
        let wasm_bin = std::fs::read(wasm_path).expect("Unable to read token actor file");

        // Set actor state
        let actor_state = TokenState::new(&blockstore).unwrap(); // TODO: this should probably not be exported from the package
        let state_cid = tester.set_state(&actor_state).unwrap();

        tester
            .set_actor_from_bin(&wasm_bin, state_cid, actor_address, TokenAmount::zero())
            .unwrap();

        // Instantiate machine
        tester.instantiate_machine(DummyExterns).unwrap();

        // Helper to simplify sending messages
        let mut sequence = 0u64;
        let mut call_method = |from, to, method_num, params| {
            let message = Message {
                from,
                to,
                gas_limit: i64::MAX,
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
        let ret_val = call_method(actor[0].1, actor_address, method_hash!("Constructor"), None);
        println!("token actor constructor return data: {:?}", &ret_val);

        let method_number =
            u64::from_le_bytes([test_case.0, test_case.1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // Create the accounts in the balance hamt
        let ret_val = call_method(actor[0].1, actor_address, method_number, None);
        println!("creating_balances {:?}", &ret_val);

        let method_number =
            u64::from_le_bytes([test_case.0, test_case.1, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // Simulate transfers
        let ret_val = call_method(actor[0].1, actor_address, method_number, None);
        println!("simulating_transfers {:?}", &ret_val);

        println!("test case: {:?}", test_case);
        println!("Gas used {}", ret_val.msg_receipt.gas_used);
        println!("=======================================================================")
    }
}
