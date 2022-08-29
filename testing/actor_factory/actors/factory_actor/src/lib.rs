use std::f32::consts::E;

use cid::Cid;
use frc42_dispatch::match_method;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_ipld_encoding::DAG_CBOR;
use fvm_ipld_encoding::{de::DeserializeOwned, RawBytes};
use fvm_sdk as sdk;
use fvm_shared::receipt::Receipt;
use fvm_shared::{address::Address, bigint::Zero, econ::TokenAmount, error::ExitCode};
use sdk::NO_DATA_BLOCK_ID;

/// Grab the incoming parameters and convert from RawBytes to deserialized struct
pub fn deserialize_params<O: DeserializeOwned>(params: u32) -> O {
    let params = sdk::message::params_raw(params).unwrap().1;
    let params = RawBytes::new(params);
    params.deserialize().unwrap()
}

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct CreateFrc42Params {
    pub constructor_params: RawBytes,
    /// ideally we'd pass the code cid directly in rather than an actor-instance's address but the ref-fvm integration test
    /// doesn't expose the code cid to us
    pub code_cid: Cid,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug)]
pub struct CreateFrc42Return {
    pub receipt: Receipt,
    pub exec_return: ExecReturn,
}

/// Following structs taken from filecoin-project/builtin-actors/actors/init/src/types.rs
/// Init actor Exec Params
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct ExecParams {
    pub code_cid: Cid,
    pub constructor_params: RawBytes,
}

/// Init actor Exec Return value
#[derive(Serialize_tuple, Deserialize_tuple, Debug)]
pub struct ExecReturn {
    /// ID based address for created actor
    pub id_address: Address,
    /// Reorg safe address for actor
    pub robust_address: Address,
}

#[no_mangle]
fn invoke(id: u32) -> u32 {
    let method_num = sdk::message::method_number();
    match_method!(method_num, {
        "Constructor" => {
            // this is a stateless actor so constructor does nothing
            NO_DATA_BLOCK_ID
        },
        "CreateFrc42Token" => {
            let params = sdk::message::params_raw(id).unwrap().1;
            let params = RawBytes::new(params);
            let params = params.deserialize::<CreateFrc42Params>().unwrap();

            let receipt = create_actor(params);
            let bytes = fvm_ipld_encoding::to_vec(&receipt).unwrap();
            sdk::ipld::put_block(DAG_CBOR, bytes.as_slice()).unwrap()
        },
        _ => {
            sdk::vm::abort(
                ExitCode::USR_UNHANDLED_MESSAGE.value(),
                Some("Unknown method number"),
            );
        }
    })
}

fn create_actor(params: CreateFrc42Params) -> Result<CreateFrc42Return, String> {
    // this is a slightly weird way to get the code cid
    // let address = params.actor_address;
    // let cid = sdk::actor::get_actor_code_cid(&address).unwrap();

    // message the builtin-init actor to create a new actor
    let init_actor_exec_params =
        ExecParams { code_cid: params.code_cid, constructor_params: params.constructor_params };
    let bytes = match fvm_ipld_encoding::to_vec(&init_actor_exec_params) {
        Ok(bytes) => bytes,
        Err(e) => return Err(format!("Failed to serialize init actor exec params {}", e)),
    };
    let receipt = match sdk::send::send(&Address::new_id(1), 2, bytes.into(), TokenAmount::zero()) {
        Ok(rec) => {
            if !rec.exit_code.is_success() {
                return Err(format!("Failed calling exec on init actor {:#?}", rec));
            } else {
                rec
            }
        }

        Err(e) => return Err(format!("Failed calling exec on init actor {}", e)),
    };

    // this data is duplicated but it's convenient to extract the underlying return value here
    let exec_return = receipt.return_data.clone();
    let exec_return = match exec_return.deserialize::<ExecReturn>() {
        Ok(ret) => ret,
        Err(e) => return Err(format!("Failed deserializing stuff from init actor {}", e)),
    };
    Ok(CreateFrc42Return { receipt, exec_return })
}
