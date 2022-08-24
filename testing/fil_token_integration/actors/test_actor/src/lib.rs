use fil_fungible_token::receiver::types::TokenReceivedParams;
use fvm_dispatch::{match_method, method_hash};
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_ipld_encoding::{DAG_CBOR, de::DeserializeOwned, ser::Serialize, Cbor, RawBytes};
use fvm_sdk as sdk;
use fvm_shared::{MethodNum, address::Address, bigint::Zero, econ::TokenAmount, error::ExitCode, receipt::Receipt};
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
    sdk::vm::abort(ExitCode::USR_ILLEGAL_ARGUMENT.value(), Some(message.as_str()));
}

fn send<S: Serialize>(to: &Address, method: MethodNum, params: &S) -> Result<Receipt, String>  {
    let params = RawBytes::new(fvm_ipld_encoding::to_vec(&params).unwrap());
    sdk::send::send(to, method, params, TokenAmount::zero()).map_err(|err| format!("send error: {:?}", err))
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
            let address: Address = deserialize_params(input);
            let recv_params =
                TokenReceivedParams { 
                    operator: 10000u64,
                    from: 10000u64,
                    to: 10010u64,
                    amount: TokenAmount::from(100),
                    data: Default::default(),
                };
            let params = RawBytes::serialize(recv_params).unwrap();
            let ret = send(&address, method_hash!("TokensReceived"), &params);
            let ret_text = ret.as_ref().map_or_else(|_| String::from("[error]"), |r| r.return_data.deserialize::<String>().unwrap());
            let ret = format!("result {:?}\n{}", ret, ret_text);
            return_ipld(&ret)
        },
        "TakeString" => {
            let address: Address = deserialize_params(input);
            let params = RawBytes::serialize(String::from("i am a string")).unwrap();
            let ret = send(&address, method_hash!("TakeString"), &params);
            let ret_text = ret.as_ref().map_or_else(|_| String::from("[error]"), |r| r.return_data.deserialize::<String>().unwrap());
            let ret = format!("result {:?}\n{}", ret, ret_text);
            return_ipld(&ret)
        },
        "TakeInteger" => {
            let address: Address = deserialize_params(input);
            let params = RawBytes::serialize(12345678u64).unwrap();
            let ret = send(&address, method_hash!("TakeInteger"), &params);
            let ret_text = ret.as_ref().map_or_else(|_| String::from("[error]"), |r| r.return_data.deserialize::<String>().unwrap());
            let ret = format!("result {:?}\n{}", ret, ret_text);
            return_ipld(&ret)
        },
        "TakeStruct" => {
            let address: Address = deserialize_params(input);
            let recv_params =
                TestStruct { 
                    number: 12345678u64,
                    data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                };
            let params = RawBytes::serialize(recv_params).unwrap();
            let ret = send(&address, method_hash!("TakeStruct"), &params);
            let ret_text = ret.as_ref().map_or_else(|_| String::from("[error]"), |r| r.return_data.deserialize::<String>().unwrap());
            let ret = format!("result {:?}\n{}", ret, ret_text);
            return_ipld(&ret)
        },
        _ => {
            sdk::vm::abort(
                ExitCode::USR_ILLEGAL_ARGUMENT.value(),
                Some("Unknown method number"),
            );
        }
    })
}
