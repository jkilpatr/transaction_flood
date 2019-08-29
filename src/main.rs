use deep_space::address::Address;
use deep_space::coin::Coin;
use deep_space::msg::Msg;
use deep_space::msg::SendMsg;
use deep_space::private_key::PrivateKey;
use deep_space::stdfee::StdFee;
use deep_space::stdsignmsg::StdSignMsg;
use docopt::Docopt;
use reqwest::Client;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
#[macro_use]
extern crate serde_derive;

#[derive(Debug, Deserialize)]
struct Args {
    flag_key: String,
    flag_fullnode: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct CosmosAccountInfoWrapper {
    #[serde(rename = "type")]
    account_type: String,
    value: CosmosAccountInfo
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct CosmosAccountInfo {
    account_number: String,
    coins: Vec<Coin>, 
    sequence: String,
}


fn main() {
    let usage = format!(
        "Usage: transaction-flood --key=<key> --fullnode=<fullnode>
Options:
    --key=<key>                           Private key to use to send transactions
    --fullnode=<fullnode>                 The fullnode used to send the transactions
About:
    Written By: Justin Kilpatrick (justin@althea.net)
    Version {}",
        env!("CARGO_PKG_VERSION"),
    );
    let client = Client::new();
    let args: Args = Docopt::new(usage)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    let key = args.flag_key;
    let fullnode = args.flag_fullnode;

    println!(r#"Private key secret="{}""#, key);
    let private_key = PrivateKey::from_secret(key.as_bytes());
    let public_key = private_key.to_public_key().unwrap();
    let address = public_key.to_address().unwrap();
    // Print some diagnostics
    println!("Address: {}", address.to_bech32("cosmos").unwrap());
    println!("Public key: {}", public_key.to_bech32("cosmospub").unwrap());

    let res:CosmosAccountInfoWrapper = client
        .get(&format!(
            "{}/auth/accounts/{}",
            fullnode,
            address.to_bech32("cosmos").unwrap()
        ))
        .send()
        .unwrap()
        .json().unwrap();

    let std_sign_msg = StdSignMsg {
        chain_id: "altheatest4".to_string(),
        account_number: res.value.account_number.parse().unwrap(),
        sequence: res.value.sequence.parse().unwrap(),
        fee: StdFee {
            amount: vec![Coin {
                denom: "ualtg".to_string(),
                amount: 42u32.into(),
            }],
            gas: 200_000u64.into(),
        },
        msgs: vec![Msg::SendMsg(SendMsg {
            from_address: address,
            to_address: Address::from_bech32(
                "cosmos1x47p0tjylnsmxyy33q4mhqtxhzwk5009fh35sq".to_string(),
            )
            .unwrap(),
            amount: vec![Coin {
                denom: "ualtg".to_string(),
                amount: 500u32.into(),
            }],
        })],
        memo: "hello from Curiosity".to_string(),
    };

    let tx = private_key.sign_std_msg(std_sign_msg).unwrap();
    let s = serde_json::to_string_pretty(&tx).unwrap();
    let mut file = File::create("signed_msg.json").unwrap();
    file.write_all(s.as_bytes()).unwrap();
    println!("{:?}", tx);
    let res = client
        .post(&format!("{}/txs", fullnode))
        .json(&tx)
        .send()
        .unwrap()
        .text();

    println!("{:?}", res);
}
