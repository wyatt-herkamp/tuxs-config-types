macro_rules! serde_via_string_types {
    (
        $type:ty
    ) => {
        const _: () = {
            impl serde::Serialize for $type {
                fn serialize<S>(
                    &self,
                    serializer: S,
                ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
                where
                    S: serde::Serializer,
                {
                    self.to_string().serialize(serializer)
                }
            }

            impl<'de> serde::Deserialize<'de> for $type {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    let s = String::deserialize(deserializer)?;
                    Self::from_str(&s).map_err(serde::de::Error::custom)
                }
            }
        };
    };
}
pub(crate) use serde_via_string_types;

macro_rules! extend_string_from_and_to {
    ($type:ty, $error:ty) => {
        const _: () = {
            impl std::convert::TryFrom<std::string::String> for $type {
                type Error = $error;

                fn try_from(value: std::string::String) -> std::result::Result<Self, Self::Error> {
                    <$type>::from_str(&value)
                }
            }
            impl std::convert::TryFrom<&str> for $type {
                type Error = $error;

                fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
                    <$type>::from_str(value)
                }
            }
            impl std::convert::From<$type> for std::string::String {
                fn from(value: $type) -> std::string::String {
                    value.to_string()
                }
            }
        };
    };
}
pub(crate) use extend_string_from_and_to;
