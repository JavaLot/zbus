#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zvariant::{
    dict_utils::{opt_value, value},
    serialized::{Context, Format},
    OwnedValue, Type, Value, LE,
};

#[test]
fn derive_unit_struct() {
    #[derive(Type)]
    struct FooF(f64);

    assert_eq!(FooF::SIGNATURE, "d")
}

#[test]
fn derive_struct() {
    #[derive(Type)]
    struct TestStruct {
        name: String,
        age: u8,
        blob: Vec<u8>,
    }

    assert_eq!(TestStruct::SIGNATURE, "(syay)")
}

#[test]
fn derive_enum() {
    #[repr(u32)]
    #[derive(Type)]
    enum RequestNameFlags {
        AllowReplacement = 0x01,
        ReplaceExisting = 0x02,
        DoNotQueue = 0x04,
    }

    assert_eq!(RequestNameFlags::SIGNATURE, "u")
}

#[test]
fn derive_dict() {
    #[derive(Serialize, Deserialize, Type, Default)]
    #[zvariant(signature = "a{sv}")]
    #[serde(deny_unknown_fields, rename_all = "camelCase", default)]
    struct Test {
        #[serde(with = "opt_value", skip_serializing_if = "Option::is_none")]
        field_a: Option<u32>,
        #[serde(with = "value", rename = "field-b")]
        field_b: String,
        #[serde(with = "value")]
        field_c: Vec<u8>,
    }

    let test = Test {
        field_a: Some(1),
        field_b: "foo".to_string(),
        field_c: vec![1, 2, 3],
    };

    let ctxt = Context::new(Format::DBus, LE, 0);
    let serialized = zvariant::to_bytes(ctxt, &test).unwrap();
    let deserialized: HashMap<String, OwnedValue> = serialized.deserialize().unwrap().0;

    assert_eq!(
        deserialized["fieldA"],
        Value::from(1u32).try_into().unwrap()
    );
    assert_eq!(
        deserialized["field-b"],
        Value::from("foo").try_into().unwrap()
    );
    assert_eq!(
        deserialized["fieldC"],
        Value::from(&[1u8, 2, 3][..]).try_into().unwrap()
    );

    let serialized = zvariant::to_bytes(ctxt, &deserialized).unwrap();
    let deserialized: Test = serialized.deserialize().unwrap().0;

    assert_eq!(deserialized.field_a, Some(1u32));
    assert_eq!(deserialized.field_b.as_str(), "foo");
    assert_eq!(deserialized.field_c.as_slice(), &[1u8, 2, 3][..]);

    assert_eq!(Test::SIGNATURE, "a{sv}")
}

#[test]
#[ignore]
fn issues_311() {
    // Issue 311: Value macro not able to handle Option in Dict.
    //
    // org.freedesktop.ModemManager1.Modem.Signal props are a dict with optional values depending on
    // the property you read.
    #[derive(Debug, Type, Deserialize, OwnedValue, Value, Default)]
    #[zbus(signature = "dict")]
    #[serde(deny_unknown_fields, default)]
    pub struct SignalInfo {
        #[serde(with = "opt_value")]
        pub rssi: Option<i32>,
        #[serde(with = "opt_value")]
        pub ecio: Option<i32>,
        #[serde(with = "opt_value")]
        pub io: Option<i32>,
        #[serde(with = "opt_value")]
        pub sinr: Option<i32>,
    }
}

#[test]
#[ignore]
fn issues_1252() {
    // Issue 1252: Naming a field `key` in a dict struct causes a conflict with variables created by
    // `DeserializeDict` macro, ending up with a strange error.
    #[derive(Type, Deserialize)]
    #[zvariant(signature = "a{sv}")]
    pub struct OwnedProperties {
        #[serde(with = "value")]
        key: String,
        #[serde(with = "value")]
        val: OwnedValue,
    }
}
