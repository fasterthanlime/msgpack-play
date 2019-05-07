use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, SerializeSeq, Serializer};

#[derive(Debug)]
enum Message {
    Request(Request),
    #[allow(unused)]
    Response(Response),
}

impl Serialize for Message {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Message::Request(req) => {
                let mut seq = s.serialize_seq(Some(4))?;
                seq.serialize_element(&0)?;
                seq.serialize_element(&req.id)?;
                seq.serialize_element(req.params.method())?;
                seq.serialize_element(&req.params)?;
                seq.end()
            }
            _ => panic!("unknown variant"),
        }
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}

#[derive(Debug)]
struct Request {
    parent: Option<u32>,
    id: u32,
    params: Params,
}

mod profile {
    #[derive(Debug)]
    pub enum Params {
        LoginWithPassword(login_with_password::Params),
    }

    impl serde::ser::Serialize for Params {
        fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
        where
            S: serde::ser::Serializer,
        {
            match self {
                Params::LoginWithPassword(x) => x.serialize(s),
            }
        }
    }

    impl super::ParamsLike for Params {
        fn method(&self) -> &'static str {
            match self {
                Params::LoginWithPassword(_) => "Profile.LoginWithPassword",
            }
        }
    }

    pub mod login_with_password {
        use serde_derive::*;

        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
        pub struct Params {
            pub username: String,
            pub password: String,
        }

        impl Into<super::super::Params> for Params {
            fn into(self) -> super::super::Params {
                super::super::Params::Profile(super::Params::LoginWithPassword(self))
            }
        }
    }
}

#[derive(Debug)]
enum Params {
    Profile(profile::Params),
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
            Params::Profile(x) => x.serialize(s),
        }
    }
}

impl ParamsLike for Params {
    fn method(&self) -> &'static str {
        match self {
            Params::Profile(x) => x.method(),
        }
    }
}

#[derive(Debug)]
struct Response {
    id: u32,
    error: String,
}

fn main() {
    let msg = Message::Request(Request {
        parent: None,
        id: 42069,
        params: profile::login_with_password::Params {
            username: "john".into(),
            password: "hunter2".into(),
        }
        .into(),
    });

    let mut buf: Vec<u8> = Vec::new();
    msg.serialize(&mut rmp_serde::Serializer::new_named(&mut buf))
        .unwrap();

    {
        use std::io::Write;
        std::fs::File::create("./buf.bin")
            .unwrap()
            .write_all(&buf)
            .unwrap();
    }

    println!("Structure: ");
    dump_as_json(&mut &buf[..]);

    let msg2: Message = rmp_serde::decode::from_slice(&buf[..]).unwrap();
    if bitcompare(&msg2, &msg) {
        println!("serde cycle matches");
    } else {
        panic!("msg should be equal after serde")
    }
}

fn bitcompare<T, U>(t: &T, u: &U) -> bool {
    unsafe {
        use std::mem::{size_of, transmute};
        use std::slice::from_raw_parts as transgress;
        let t: &[u8] = transgress(transmute(t), size_of::<T>());
        let u: &[u8] = transgress(transmute(u), size_of::<U>());
        t == u
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
