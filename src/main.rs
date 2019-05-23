use fasthash::{sea::Hash64, FastHash};
use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::fmt;

#[derive(Debug)]
enum Message {
    Request {
        parent: Option<u32>,
        id: u32,
        params: Params,
    },
    #[allow(unused)]
    Response {
        id: u32,
        error: Option<String>,
        results: Results,
    },
}

impl Serialize for Message {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Message::Request { id, params, .. } => {
                let mut seq = s.serialize_seq(Some(4))?;
                seq.serialize_element(&0)?;
                seq.serialize_element(&id)?;
                seq.serialize_element(params.method())?;
                seq.serialize_element(&params)?;
                seq.end()
            }
            _ => panic!("unknown variant"),
        }
    }
}

struct MessageVisitor;

impl<'de> Visitor<'de> for MessageVisitor {
    type Value = Message;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "a valid msgpack-RPC message (ie. a sequence)")
    }

    fn visit_seq<S>(self, mut access: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        use serde::de::Error;
        let missing = |field: &str| -> S::Error {
            S::Error::custom(format!("invalid msgpack-RPC message: missing {}", field))
        };

        let typ = access.next_element::<u32>()?.ok_or(missing("type"))?;

        match typ {
            0 => {
                // let's parse a request
                let id = access.next_element::<u32>()?.ok_or(missing("id"))?;
                let method = access.next_element::<String>()?.ok_or(missing("method"))?;
                let params = Params::deserialize(&method, &mut access)?.ok_or(missing("params"))?;

                Ok(Message::Request {
                    parent: None,
                    id,
                    params,
                })
            }
            _ => unimplemented!(),
        }
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_seq(MessageVisitor {})
    }
}

mod profile {
    pub mod login_with_password {
        use serde_derive::*;
        pub const HASH: u64 = 14049389558238484926;

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Params {
            pub username: String,
            pub password: String,
            #[serde(with = "serde_bytes")]
            pub bytes: Vec<u8>,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Results {
            pub ok: bool,
        }
    }

    pub mod login_with_token {
        use serde_derive::*;
        pub const HASH: u64 = 4196540711449377264;

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Params {
            pub token: String,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Results {
            pub ok: bool,
        }
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types, unused)]
enum Params {
    Profile_LoginWithPassword(profile::login_with_password::Params),
    Profile_LoginWithToken(profile::login_with_token::Params),
}

#[derive(Debug)]
#[allow(non_camel_case_types, unused)]
enum Results {
    Profile_LoginWithPassword(profile::login_with_password::Results),
    Profile_LoginWithToken(profile::login_with_token::Results),
}

pub trait ParamsLike: serde::Serialize + std::fmt::Debug {
    fn method(&self) -> &'static str;
}

impl Serialize for Params {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Params::Profile_LoginWithPassword(x) => x.serialize(s),
            Params::Profile_LoginWithToken(x) => x.serialize(s),
        }
    }
}

impl Params {
    fn deserialize<'de, S: SeqAccess<'de>>(
        method: &str,
        access: &mut S,
    ) -> Result<Option<Self>, S::Error> {
        use serde::de::Error;
        let hash = Hash64::hash(method);
        match hash {
            profile::login_with_password::HASH => Ok(access
                .next_element::<profile::login_with_password::Params>()?
                .map(Params::Profile_LoginWithPassword)),
            profile::login_with_token::HASH => Ok(access
                .next_element::<profile::login_with_token::Params>()?
                .map(Params::Profile_LoginWithToken)),
            _ => Err(S::Error::custom(format!(
                "unknown method {} (hash = {})",
                method, hash
            ))),
        }
    }
}

impl ParamsLike for Params {
    fn method(&self) -> &'static str {
        match self {
            Params::Profile_LoginWithPassword(_) => "Profile.LoginWithPassword",
            Params::Profile_LoginWithToken(_) => "Profile.LoginWithToken",
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
struct Response {
    id: u32,
    error: String,
}

fn cycle(msg: Message) {
    let mut buf: Vec<u8> = Vec::new();
    msg.serialize(&mut rmp_serde::Serializer::new_named(&mut buf))
        .unwrap();

    {
        use std::io::Write;
        let path = "./buf.bin";
        std::fs::File::create(path)
            .unwrap()
            .write_all(&buf)
            .unwrap();
        println!("Written to {}", path);
    }

    println!("{:#?}", msg);

    if false {
        println!("Structure: ");
        dump_as_json(&mut &buf[..]);
    }

    let msg2: Message = rmp_serde::decode::from_slice(&buf[..]).unwrap();
    let mut buf2: Vec<u8> = Vec::new();
    msg2.serialize(&mut rmp_serde::Serializer::new_named(&mut buf2))
        .unwrap();

    if buf == buf2 {
        println!("serde cycle matches");
    } else {
        panic!(
            "serde cycle doesn't match.\nexpected {:#?}\ngot: {:#?}",
            buf, buf2
        )
    }
}

fn main() {
    if false {
        cycle(Message::Request {
            parent: None,
            id: 548,
            params: Params::Profile_LoginWithToken(profile::login_with_token::Params {
                token: "alrighty".into(),
            }),
        });
    }

    if false {
        cycle(Message::Request {
            parent: None,
            id: 127,
            params: Params::Profile_LoginWithPassword(profile::login_with_password::Params {
                username: "john".into(),
                password: "hunter2".into(),
                bytes: vec![0x31, 0x00, 0x12, 0x24],
            }),
        });
    }

    {
        use serde_derive::*;

        #[derive(Serialize)]
        struct S1 {
            t: (),
        }

        #[derive(Serialize)]
        struct S2 {
            t: S22,
        }

        #[derive(Serialize)]
        struct S22 {}

        {
            let msg = S1 { t: () };
            let mut buf: Vec<u8> = Vec::new();
            msg.serialize(&mut rmp_serde::Serializer::new_named(&mut buf))
                .unwrap();
            println!("{:#?}", buf);
        }

        {
            let msg = S2 { t: S22 {} };
            let mut buf: Vec<u8> = Vec::new();
            msg.serialize(&mut rmp_serde::Serializer::new_named(&mut buf))
                .unwrap();
            println!("{:#?}", buf);
        }
    }
}

fn dump_as_json<R>(input: R)
where
    R: std::io::Read,
{
    let mut out: Vec<u8> = Vec::new();
    let mut d = rmp_serde::Deserializer::new(input);
    let mut s = serde_json::Serializer::pretty(&mut out);
    serde_transcode::transcode(&mut d, &mut s).unwrap();
    let out = String::from_utf8_lossy(&out);
    println!("{}", out);
}
