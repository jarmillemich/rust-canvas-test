use bevy::reflect::impl_reflect_value;
use fixed::{types::extra::U12, FixedI64};
use serde::{Deserialize, Serialize};

custom_derive! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, NewtypeAdd, NewtypeSub, NewtypeMul, NewtypeDiv, NewtypeSubAssign, NewtypeAddAssign, NewtypeMulAssign, NewtypeDivAssign)]
    pub struct FixedPoint(FixedI64<U12>);
}

impl_reflect_value!(FixedPoint);

impl Serialize for FixedPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for FixedPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize(deserializer)
    }
}

impl FixedPoint {
    pub fn from_num<T: fixed::traits::ToFixed>(num: T) -> Self {
        Self(FixedI64::<U12>::from_num::<T>(num))
    }

    pub fn to_num<T: fixed::traits::FromFixed>(&self) -> T {
        self.0.to_num::<T>()
    }

    pub fn to_be_bytes(&self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    pub fn from_be_bytes(bytes: [u8; 8]) -> Self {
        Self(FixedI64::<U12>::from_be_bytes(bytes))
    }
}

/// For serde to serialize a FixedPoint
/// Annotate any serializable FixedPoint fields with #[serde(with = "crate::fixed_point")]
pub fn serialize<S>(fp: &FixedPoint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    println!("It me, custom ser!");
    let bytes = fp.to_be_bytes();
    let encoded = base64::encode(&bytes);
    serializer.serialize_str(encoded.as_str())
}

/// For serde to deserialize a FixedPoint
/// Annotate any deserializable FixedPoint fields with #[serde(with = "crate::fixed_point")]
pub fn deserialize<'de, D>(deserializer: D) -> Result<FixedPoint, D::Error>
where
    D: serde::Deserializer<'de>,
{
    println!("It me, custom de!");
    let encoded = String::deserialize(deserializer)?;
    let bytes = base64::decode(encoded.as_str()).unwrap();
    println!("It me, custom de2!");
    Ok(FixedPoint::from_be_bytes(bytes[0..8].try_into().unwrap()))
}

// impl Reflect for FixedPoint {
//     fn type_name(&self) -> &str {
//         "FixedPoint"
//     }

//     fn get_type_info(&self) -> &'static bevy::reflect::TypeInfo {
//         todo!()
//     }

//     fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
//         todo!()
//     }

//     fn as_any(&self) -> &dyn std::any::Any {
//         todo!()
//     }

//     fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//         todo!()
//     }

//     fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
//         todo!()
//     }

//     fn as_reflect(&self) -> &dyn Reflect {
//         todo!()
//     }

//     fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
//         todo!()
//     }

//     fn apply(&mut self, value: &dyn Reflect) {
//         todo!()
//     }

//     fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
//         todo!()
//     }

//     fn reflect_ref(&self) -> bevy::reflect::ReflectRef {
//         bevy::reflect::ReflectRef::Struct(&self)
//     }

//     fn reflect_mut(&mut self) -> bevy::reflect::ReflectMut {
//         todo!()
//     }

//     fn reflect_owned(self: Box<Self>) -> bevy::reflect::ReflectOwned {
//         todo!()
//     }

//     fn clone_value(&self) -> Box<dyn Reflect> {
//         Box::new(self.clone())
//     }
// }
