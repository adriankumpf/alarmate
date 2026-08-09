use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::{self, Deserializer, Visitor};

/// Implement `Serialize` and `Deserialize` for one or more `#[repr(u8)]` enums
/// that derive `AsRefStr`, `EnumString` and `TryFromPrimitive`.
///
/// Values are written as the variant name and read back from either the
/// discriminant — which is what the panel sends — or the variant name, so the
/// two directions round-trip.
macro_rules! impl_enum_serde {
    ($($T:ty),+ $(,)?) => { $(
        impl serde::Serialize for $T {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(self.as_ref())
            }
        }

        impl<'de> serde::Deserialize<'de> for $T {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                $crate::utils::deserialize_enum(d)
            }
        }
    )+ };
}

pub(crate) use impl_enum_serde;

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
                // Not a discriminant, so fall back to the variant name — that is
                // what our own `Serialize` impl writes.
                Err(_) => s.parse().map_err(E::custom),
            }
        }
    }

    deserializer.deserialize_any(EnumVisitor(PhantomData))
}
