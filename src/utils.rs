/// Creates an enum that can be deserialized from a number
#[macro_export]
macro_rules! enum_number {
    (
        $(#[$enum_attr:meta])*
        $name:ident {
            $(#[$attr:meta] $variant:ident = $value:expr, )*
        }
    ) => {
        $(#[$enum_attr])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[cfg_attr(feature = "build-binary", derive(clap::ValueEnum))]
        pub enum $name {
            $(#[$attr] $variant = $value,)*
        }

                impl ::std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self,Self::Err> {
                match s {
                    $(stringify!($variant) |
                    _ if s.eq_ignore_ascii_case(stringify!($variant)) => Ok($name::$variant),)+
                    _                => Err({
                                            let v = vec![
                                                $(stringify!($variant),)+
                                            ];
                                            format!("valid values:{}",
                                                v.iter().fold(String::new(), |a, i| {
                                                    a + &format!(" {}", i)[..]
                                                }))
                                        })
                }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match *self {
                    $($name::$variant => write!(f, stringify!($variant)),)+
                }
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(match *self {
                    $( $name::$variant => stringify!($variant), )*
                })

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
