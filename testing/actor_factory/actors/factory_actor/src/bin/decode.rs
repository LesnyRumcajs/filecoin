use base64;
use factory_actor::CreateFrc42Return;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::bigint::bigint_ser;
use fvm_shared::{address::Address, econ::TokenAmount};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};

#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug)]
pub struct MintParams {
    pub initial_owner: Address,
    #[serde(with = "bigint_ser")]
    pub amount: TokenAmount,
}

fn main() {
    let input: &str =
        "goMAWBuCQwDqB1UCgfjJ3ZOCNp6QjnIs6lUnh32J0fwAgkMA6gdVAoH4yd2TgjaekI5yLOpVJ4d9idH8";

    let vec = base64::decode(input).unwrap();

    let bytes = RawBytes::from(vec);
    let params = bytes.deserialize::<CreateFrc42Return>().unwrap();
    println!("{:#?}", params);

    let mint_params =
        MintParams { initial_owner: Address::new_id(1006), amount: TokenAmount::from(1000) };

    let serialized = fvm_ipld_encoding::to_vec(&mint_params).unwrap();
    let res = base64::encode(&serialized);
    println!("{:?}", res)
}
