use fixed::{types::extra::U12, FixedI64};
use serde::Deserialize;

pub type FixedPoint = FixedI64<U12>;

/// For serde to serialize a FixedPoint
/// Annotate any serializable FixedPoint fields with #[serde(with = "crate::fixed_point")]
pub fn serialize<S>(fp: &FixedPoint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_bytes(&fp.to_be_bytes())
}

/// For serde to deserialize a FixedPoint
/// Annotate any deserializable FixedPoint fields with #[serde(with = "crate::fixed_point")]
pub fn deserialize<'de, D>(deserializer: D) -> Result<FixedPoint, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let bytes = <[u8; 8]>::deserialize(deserializer)?;
    Ok(FixedPoint::from_be_bytes(bytes))
}
