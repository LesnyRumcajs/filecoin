use fil_fungible_token::receiver::types::TokenReceivedParams;
use fvm_dispatch::match_method;
//use fvm_ipld_encoding::{de::DeserializeOwned, Cbor, RawBytes};
use fvm_ipld_encoding::{DAG_CBOR, de::DeserializeOwned, ser::Serialize, Cbor, RawBytes};
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_sdk as sdk;
use fvm_shared::error::ExitCode;
use sdk::NO_DATA_BLOCK_ID;

/// Grab the incoming parameters and convert from RawBytes to deserialized struct
pub fn deserialize_params<O: DeserializeOwned>(params: u32) -> O {
    let params = sdk::message::params_raw(params).unwrap().1;
    let params = RawBytes::new(params);
    params.deserialize().unwrap()
}

#[derive(Clone, Debug, Deserialize_tuple, Serialize_tuple)]
struct TestStruct {
    number: u64,
    data: Vec<u8>,
}
impl Cbor for TestStruct {}

pub fn deserialize_params_abort<O: DeserializeOwned + std::fmt::Debug>(params: u32) -> O {
    let params = sdk::message::params_raw(params).unwrap().1;
    let params = RawBytes::new(params);
    let params = params.deserialize::<O>();
    let message = format!("params was {:?}", params);
    sdk::vm::abort(
        ExitCode::USR_ILLEGAL_ARGUMENT.value(),
        Some(message.as_str()),
    );
}

fn return_ipld<T>(value: &T) -> u32
where
    T: Serialize + ?Sized,
{
    let bytes = fvm_ipld_encoding::to_vec(value).unwrap();
    sdk::ipld::put_block(DAG_CBOR, bytes.as_slice()).unwrap()
}

#[no_mangle]
fn invoke(input: u32) -> u32 {
    let method_num = sdk::message::method_number();
    match_method!(method_num, {
        "Constructor" => {
            // this is a stateless actor so constructor does nothing
            NO_DATA_BLOCK_ID
        },
        "TokensReceived" => {
            let params = sdk::message::params_raw(input).unwrap().1;
            let params = RawBytes::new(params);
            // why does the deserialise operation fail?
            // "params was RawBytes { bytes: [77, 133, 24, 100, 25, 39, 16, 25, 39, 26, 66, 0, 100, 64] }"
            // "params was Err(Error { description: \"Mismatch { expect_major: 4, byte: 77 }\", protocol: Cbor })"

            let params = params.deserialize::<TokenReceivedParams>();
            let message = format!("params was {:?}", params);
            /*sdk::vm::abort(
                ExitCode::USR_ILLEGAL_ARGUMENT.value(),
                Some(message.as_str()),
            );*/
            // TokensReceived is passed a TokenReceivedParams
            //let _params: TokenReceivedParams = deserialize_params(input);

            // decide if we care about incoming tokens or not
            // if we don't want them, abort

            //NO_DATA_BLOCK_ID
            return_ipld(&message)
        },
        "TakeString" => {
            let params: String = deserialize_params(input);
            let message = format!("params was {:?}", params);
            //NO_DATA_BLOCK_ID
            return_ipld(&message)
        },
        "TakeInteger" => {
            let params: u64 = deserialize_params(input);
            let message = format!("params was {:?}", params);
            //NO_DATA_BLOCK_ID
            return_ipld(&message)
        },
        "TakeStruct" => {
            let params: TestStruct = deserialize_params(input);
            let message = format!("params was {:?}", params);
            //NO_DATA_BLOCK_ID
            return_ipld(&message)
        },
        _ => {
            sdk::vm::abort(
                ExitCode::USR_ILLEGAL_ARGUMENT.value(),
                Some("Unknown method number"),
            );
        }
    })
}
