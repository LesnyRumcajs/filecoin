use base64;
use factory_actor::CreateFrc42Return;
use fvm_ipld_encoding::RawBytes;

fn main() {
    let input: &str =
        "goMAWBuCQwDqB1UCav1ChP2Wqo8IQXOOMCvUH9gFqoYAgkMA6gdVAmr9QoT9lqqPCEFzjjAr1B/YBaqG";

    let vec = base64::decode(input).unwrap();

    let bytes = RawBytes::from(vec);
    let params = bytes.deserialize::<CreateFrc42Return>().unwrap();
    println!("{:#?}", params);
}
