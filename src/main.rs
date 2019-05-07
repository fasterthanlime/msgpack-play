use rmp_serde::{Deserializer, Serializer};
use serde::Serialize;
use serde_derive::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type")]
enum RPCMessage {
    Request(Request),
    Response(Response),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Request {
    id: u32,
    params: Params,
}

mod profile {
    pub mod login_with_password {
        use serde_derive::*;

        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
        pub struct Params {
            pub username: String,
            pub password: String,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "method", content = "payload")]
enum Params {
    LoginWithPassword(profile::login_with_password::Params),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Response {
    id: u32,
    method: String,
    error: Option<String>,
}

fn main() {
    let msg = RPCMessage::Request(Request {
        id: 42069,
        params: Params::LoginWithPassword(profile::login_with_password::Params {
            username: "john".into(),
            password: "hunter2".into(),
        }),
    });

    let mut buf: Vec<u8> = Vec::new();
    msg.serialize(&mut Serializer::new_named(&mut buf)).unwrap();

    println!("Structure: ");
    dump_as_json(&mut &buf[..]);

    let msg2: RPCMessage = rmp_serde::decode::from_slice(&buf[..]).unwrap();
    if msg2 == msg {
        println!("serde cycle matches");
    } else {
        panic!("msg should be equal after serde")
    }
}

fn dump_as_json<R>(input: R)
where
    R: std::io::Read,
{
    let mut out: Vec<u8> = Vec::new();
    let mut d = Deserializer::new(input);
    let mut s = serde_json::Serializer::pretty(&mut out);
    serde_transcode::transcode(&mut d, &mut s).unwrap();
    let out = String::from_utf8_lossy(&out);
    println!("{}", out);
}
