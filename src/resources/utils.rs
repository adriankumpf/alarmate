macro_rules! enum_number {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_u64(*self as u64)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        formatter.write_str("positive integer or string")
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<$name, E>
                    where
                        E: ::serde::de::Error,
                    {
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom( format!("unknown {} value: {}", stringify!($name), value))), }
                    }


                    fn visit_str<E>(self, id: &str) -> Result<$name, E>
                    where
                        E: ::serde::de::Error
                    {
                        self.visit_u64(id.parse().map_err(::serde::de::Error::custom)?)
                    }

                }

                deserializer.deserialize_any(Visitor)
            }
        }
    }
}
