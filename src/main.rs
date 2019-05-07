use rmp_serde::{Deserializer, Serializer};
use serde::Serialize;
use serde_derive::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type")]
enum RPCMessage {
    #[serde(rename = "request")]
    Request(Request),
    #[serde(rename = "response")]
    Response(Response),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Request {
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<u32>,
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

        impl Into<super::super::Params> for Params {
            fn into(self) -> super::super::Params {
                super::super::Params::ProfileLoginWithPassword(self)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "method", content = "payload")]
enum Params {
    #[serde(rename = "Profile.LoginWithPassword")]
    ProfileLoginWithPassword(profile::login_with_password::Params),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Response {
    id: u32,
    error: Option<String>,
}

fn main() {
    let msg = RPCMessage::Request(Request {
        parent: None,
        id: 42069,
        params: profile::login_with_password::Params {
            username: "john".into(),
            password: "hunter2".into(),
        }
        .into(),
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
