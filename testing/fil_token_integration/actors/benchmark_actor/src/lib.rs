mod util;

use std::ops::Neg;

use fil_fungible_token::runtime::blockstore::Blockstore;
use fil_fungible_token::runtime::messaging::FvmMessenger;
use fil_fungible_token::token::Token;
use fvm_sdk as sdk;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::econ::TokenAmount;
use fvm_shared::error::ExitCode;
use sdk::NO_DATA_BLOCK_ID;

/// Conduct method dispatch. Handle input parameters and return data.
#[no_mangle]
pub fn invoke(_params: u32) -> u32 {
    std::panic::set_hook(Box::new(|info| {
        sdk::vm::abort(ExitCode::USR_ASSERTION_FAILED.value(), Some(&format!("{}", info)))
    }));

    let method_num = sdk::message::method_number();

    if method_num == 1 {
        return constructor();
    }

    let bytes = method_num.to_le_bytes();
    let accounts = bytes.get(0).unwrap();
    let token_size = bytes.get(1).unwrap();
    let method = bytes.get(2).unwrap();

    if *method == 0 {
        create_accounts(*accounts, *token_size)
    } else {
        make_transfers(*accounts, *token_size)
    }
}

fn constructor() -> u32 {
    let bs = Blockstore::default();
    let mut token_state = Token::<_, FvmMessenger>::create_state(&bs).unwrap();
    let mut token = Token::wrap(bs, FvmMessenger::default(), 1, &mut token_state);
    let cid = token.flush().unwrap();
    sdk::sself::set_root(&cid).unwrap();
    NO_DATA_BLOCK_ID
}

fn create_token_amount(exponent: u32) -> TokenAmount {
    let atto = BigInt::from(10);
    let atto = atto.pow(exponent);
    TokenAmount::from_atto(atto)
}

fn create_accounts(accounts: u8, token_amount: u8) -> u32 {
    let num_accounts = match accounts {
        1 => 10_000,
        2 => 100_000,
        3 => 1_000_000,
        4 => 10_000_000,
        _ => unreachable!(),
    };

    let token_amount = match token_amount {
        1 => create_token_amount(17),
        2 => create_token_amount(23),
        3 => create_token_amount(28),
        _ => unreachable!(),
    };

    let root_cid = sdk::sself::root().unwrap();

    let bs = Blockstore::default();
    let mut token_state = Token::<_, FvmMessenger>::load_state(&bs, &root_cid).unwrap();

    // create 10,000 accounts
    for i in 0..num_accounts {
        let addr = Address::new_id(i);
        token_state.set_balance(&bs, addr.id().unwrap(), &token_amount).unwrap();
    }
    let cid = token_state.save(&bs).unwrap();
    sdk::sself::set_root(&cid).unwrap();

    NO_DATA_BLOCK_ID
}

/// make 10 transfer groups evenly distributed amongst the accounts
const TRANSFER_GROUPS: u64 = 10;

fn make_transfers(accounts: u8, token_amount: u8) -> u32 {
    let num_accounts = match accounts {
        1 => 10_000,
        2 => 100_000,
        3 => 1_000_000,
        4 => 10_000_000,
        _ => unreachable!(),
    };

    let token_amount = match token_amount {
        1 => create_token_amount(17),
        2 => create_token_amount(23),
        3 => create_token_amount(28),
        _ => unreachable!(),
    };

    let root_cid = sdk::sself::root().unwrap();

    let bs = Blockstore::default();
    let mut token_state = Token::<_, FvmMessenger>::load_state(&bs, &root_cid).unwrap();

    let transfer_size_neg = &token_amount.clone().neg();
    let transfer_size = &token_amount;

    let group_size = num_accounts / TRANSFER_GROUPS;
    for group in 0..TRANSFER_GROUPS {
        // make 10 transfers per group
        for owner_id_offset in 0..10 {
            let owner = Address::new_id(group * group_size + owner_id_offset);
            // in each-group transfer to both nearby and far addresses
            let recipient =
                Address::new_id((owner_id_offset + group) * group_size + owner_id_offset);
            token_state.change_balance_by(&bs, owner.id().unwrap(), transfer_size_neg).unwrap();
            token_state.change_balance_by(&bs, recipient.id().unwrap(), transfer_size).unwrap();
            // simulate a single transaction by actually flushing the blockstore on each "transfer"
            token_state.save(&bs).unwrap();
        }
    }
    let cid = token_state.save(&bs).unwrap();
    sdk::sself::set_root(&cid).unwrap();

    NO_DATA_BLOCK_ID
}
