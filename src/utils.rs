use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::{self, Deserializer, Visitor};

/// Implement `Deserialize` for one or more `#[repr(u8)]` enums that derive
/// `EnumString` and `TryFromPrimitive`.
///
/// The panel sends discriminants, while `#[derive(Serialize)]` writes variant
/// names, so both have to be accepted for the two directions to round-trip.
macro_rules! impl_enum_deserialize {
    ($($T:ty),+ $(,)?) => { $(
        impl<'de> serde::Deserialize<'de> for $T {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                $crate::utils::deserialize_enum(d)
            }
        }
    )+ };
}

pub(crate) use impl_enum_deserialize;

/// Deserialize a fieldless enum from its discriminant or its variant name.
///
/// The target type must implement `TryFrom<u8>` (e.g. via
/// `num_enum::TryFromPrimitive`) and `FromStr` (e.g. via `strum::EnumString`).
pub(crate) fn deserialize_enum<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: TryFrom<u8> + FromStr,
    <T as TryFrom<u8>>::Error: Display,
    <T as FromStr>::Err: Display,
    D: Deserializer<'de>,
{
    struct EnumVisitor<T>(PhantomData<T>);

    impl<T> Visitor<'_> for EnumVisitor<T>
    where
        T: TryFrom<u8> + FromStr,
        <T as TryFrom<u8>>::Error: Display,
        <T as FromStr>::Err: Display,
    {
        type Value = T;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("an enum discriminant or variant name")
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<T, E> {
            let byte = u8::try_from(value).map_err(E::custom)?;
            T::try_from(byte).map_err(E::custom)
        }

        fn visit_str<E: de::Error>(self, s: &str) -> Result<T, E> {
            match s.parse::<u8>() {
                Ok(byte) => self.visit_u64(u64::from(byte)),
                // Not a discriminant, so fall back to the variant name.
                Err(_) => s.parse().map_err(E::custom),
            }
        }
    }

    deserializer.deserialize_any(EnumVisitor(PhantomData))
}
