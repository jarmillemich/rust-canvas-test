use bevy::reflect::impl_reflect_value;
use fixed::{types::extra::U12, FixedI64};
use serde::{Deserialize, Serialize};

custom_derive! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash, NewtypeAdd, NewtypeSub, NewtypeMul, NewtypeDiv, NewtypeSubAssign, NewtypeAddAssign, NewtypeMulAssign, NewtypeDivAssign)]
    pub struct FixedPoint(FixedI64<U12>);
}

impl_reflect_value!(FixedPoint);

impl Serialize for FixedPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use base64::{engine::general_purpose::STANDARD_NO_PAD as base64, Engine as _};

        let bytes = self.to_be_bytes();
        let encoded = base64.encode(bytes);
        serializer.serialize_str(encoded.as_str())
    }
}

impl<'de> Deserialize<'de> for FixedPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use base64::{engine::general_purpose::STANDARD_NO_PAD as base64, Engine as _};

        let encoded = String::deserialize(deserializer)?;
        let bytes = base64.decode(encoded.as_str()).unwrap();
        Ok(FixedPoint::from_be_bytes(bytes[0..8].try_into().unwrap()))
    }
}

impl FixedPoint {
    pub fn from_num<T: fixed::traits::ToFixed>(num: T) -> Self {
        Self(FixedI64::<U12>::from_num::<T>(num))
    }

    pub fn to_num<T: fixed::traits::FromFixed>(self) -> T {
        self.0.to_num::<T>()
    }

    pub fn to_be_bytes(self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    pub fn from_be_bytes(bytes: [u8; 8]) -> Self {
        Self(FixedI64::<U12>::from_be_bytes(bytes))
    }
}
