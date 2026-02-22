/// Implement `Serialize` (as variant name) and `Deserialize` (from integer or
/// numeric string) for one or more `#[repr(u8)]` enums that derive `Display`
/// and `TryFromPrimitive`.
macro_rules! impl_numeric_serde {
    ($($T:ty),+ $(,)?) => { $(
        impl serde::Serialize for $T {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(&self.to_string())
            }
        }

        impl<'de> serde::Deserialize<'de> for $T {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                $crate::utils::deserialize_numeric_enum(d)
            }
        }
    )+ };
}

/// Deserialize an enum from a numeric value (integer or numeric string).
///
/// The target type must implement `TryFrom<u8>` (e.g. via
/// `num_enum::TryFromPrimitive`) so that each discriminant maps to the
/// corresponding variant.
pub(crate) fn deserialize_numeric_enum<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: TryFrom<u8>,
    <T as TryFrom<u8>>::Error: std::fmt::Display,
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::marker::PhantomData;

    struct NumVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for NumVisitor<T>
    where
        T: TryFrom<u8>,
        <T as TryFrom<u8>>::Error: std::fmt::Display,
    {
        type Value = T;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("positive integer or string")
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<T, E> {
            let byte = u8::try_from(value).map_err(E::custom)?;
            T::try_from(byte).map_err(E::custom)
        }

        fn visit_str<E: de::Error>(self, s: &str) -> Result<T, E> {
            self.visit_u64(s.parse().map_err(de::Error::custom)?)
        }
    }

    deserializer.deserialize_any(NumVisitor(PhantomData))
}
