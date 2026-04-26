use serde::{Deserialize, Serialize};

/// Module containing serializers for API format (numeric enums)
pub mod api_format {
    use serde::{Serialize, Serializer};

    /// Serialize an enum to its numeric value for the API
    pub fn serialize_enum<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: super::ApiEnum,
    {
        serializer.serialize_i8(value.to_api_value())
    }

    /// Serialize an optional enum to its numeric value for the API
    pub fn serialize_option_enum<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: super::ApiEnum,
    {
        match value {
            Some(v) => serializer.serialize_i8(v.to_api_value()),
            None => serializer.serialize_none(),
        }
    }

    /// Serialize a vector of enums to their numeric values for the API
    pub fn serialize_vec_enum<S, T>(values: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: super::ApiEnum,
    {
        let nums: Vec<i8> = values.iter().map(|e| e.to_api_value()).collect();
        nums.serialize(serializer)
    }
}

/// Trait for enums that can be converted to API numeric values
pub trait ApiEnum: Sized {
    fn to_api_value(&self) -> i8;
    fn from_string(s: &str) -> Option<Self>;
}

// Macro to implement both numeric and string ser/de
macro_rules! impl_numeric_string_enum {
    ($enum_name:ident { $($variant:ident = $value:expr),* $(,)? }) => {
        #[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
        #[derive(Clone, Debug, PartialEq)]
        pub enum $enum_name {
            $($variant,)*
        }

        impl $enum_name {
            /// Return the static string representation of the enum variant
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant),)*
                }
            }
        }

        impl Serialize for $enum_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                // Always serialize as strings for TypeScript compatibility
                // The conversion to numbers happens in the API client
                let s = match self {
                    $(Self::$variant => stringify!($variant),)*
                };
                serializer.serialize_str(s)
            }
        }

        impl<'de> Deserialize<'de> for $enum_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $enum_name;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(concat!("a valid ", stringify!($enum_name), " value"))
                    }

                    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            $($value => Ok($enum_name::$variant),)*
                            _ => Err(E::custom(format!("invalid {} value: {}", stringify!($enum_name), value)))
                        }
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        self.visit_i64(value as i64)
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            $(stringify!($variant) => Ok($enum_name::$variant),)*
                            _ => Err(E::custom(format!("invalid {} value: {}", stringify!($enum_name), value)))
                        }
                    }
                }

                deserializer.deserialize_any(Visitor)
            }
        }

        impl ApiEnum for $enum_name {
            fn to_api_value(&self) -> i8 {
                match self {
                    $(Self::$variant => $value,)*
                }
            }

            fn from_string(s: &str) -> Option<Self> {
                match s {
                    $(stringify!($variant) => Some(Self::$variant),)*
                    _ => None
                }
            }
        }
    };
}

macro_rules! impl_profile_preference_enum {
    ($base:ident, $profile:ident, $preference:ident { $($variant:ident = $value:expr),* $(,)? }) => {
        impl_numeric_string_enum! {
            $base {
                $($variant = $value,)*
            }
        }

        #[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
        #[derive(Clone, Debug, PartialEq)]
        pub enum $profile {
            PreferNotToSay,
            $($variant,)*
        }

        impl $profile {
            /// Returns the base value for this profile option, if one exists.
            pub fn to_value(&self) -> Option<$base> {
                match self {
                    Self::PreferNotToSay => None,
                    $(Self::$variant => Some($base::$variant),)*
                }
            }

            /// Construct a profile value from a shared base value.
            pub fn from_value(value: $base) -> Self {
                match value {
                    $($base::$variant => Self::$variant,)*
                }
            }

            /// Convert this profile value into a preference variant when possible.
            pub fn into_preference(self) -> Option<$preference> {
                self.to_value().map($preference::from_value)
            }
        }

        impl From<$base> for $profile {
            fn from(value: $base) -> Self {
                Self::from_value(value)
            }
        }

        impl Serialize for $profile {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match self {
                    Self::PreferNotToSay => serializer.serialize_str("PreferNotToSay"),
                    $(Self::$variant => serializer.serialize_str(stringify!($variant)),)*
                }
            }
        }

        impl<'de> Deserialize<'de> for $profile {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $profile;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(concat!("a valid ", stringify!($profile), " value"))
                    }

                    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            0 => Ok($profile::PreferNotToSay),
                            $($value => Ok($profile::$variant),)*
                            _ => Err(E::custom(format!("invalid {} value: {}", stringify!($profile), value))),
                        }
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        self.visit_i64(value as i64)
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "PreferNotToSay" => Ok($profile::PreferNotToSay),
                            $(stringify!($variant) => Ok($profile::$variant),)*
                            _ => Err(E::custom(format!("invalid {} value: {}", stringify!($profile), value))),
                        }
                    }
                }

                deserializer.deserialize_any(Visitor)
            }
        }

        impl ApiEnum for $profile {
            fn to_api_value(&self) -> i8 {
                match self {
                    Self::PreferNotToSay => 0,
                    $(Self::$variant => $value,)*
                }
            }

            fn from_string(s: &str) -> Option<Self> {
                match s {
                    "PreferNotToSay" => Some(Self::PreferNotToSay),
                    $(stringify!($variant) => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }

        #[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
        #[derive(Clone, Debug, PartialEq)]
        pub enum $preference {
            OpenToAll,
            $($variant,)*
        }

        impl $preference {
            /// Returns the base value for this preference option, if one exists.
            pub fn to_value(&self) -> Option<$base> {
                match self {
                    Self::OpenToAll => None,
                    $(Self::$variant => Some($base::$variant),)*
                }
            }

            /// Construct a preference value from a shared base value.
            pub fn from_value(value: $base) -> Self {
                match value {
                    $($base::$variant => Self::$variant,)*
                }
            }

            /// Convert this preference value into a profile variant when possible.
            pub fn into_profile(self) -> Option<$profile> {
                self.to_value().map($profile::from_value)
            }
        }

        impl From<$base> for $preference {
            fn from(value: $base) -> Self {
                Self::from_value(value)
            }
        }

        impl Serialize for $preference {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match self {
                    Self::OpenToAll => serializer.serialize_str("OpenToAll"),
                    $(Self::$variant => serializer.serialize_str(stringify!($variant)),)*
                }
            }
        }

        impl<'de> Deserialize<'de> for $preference {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $preference;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(concat!("a valid ", stringify!($preference), " value"))
                    }

                    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            -1 => Ok($preference::OpenToAll),
                            $($value => Ok($preference::$variant),)*
                            _ => Err(E::custom(format!("invalid {} value: {}", stringify!($preference), value))),
                        }
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        self.visit_i64(value as i64)
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "OpenToAll" => Ok($preference::OpenToAll),
                            $(stringify!($variant) => Ok($preference::$variant),)*
                            _ => Err(E::custom(format!("invalid {} value: {}", stringify!($preference), value))),
                        }
                    }
                }

                deserializer.deserialize_any(Visitor)
            }
        }

        impl ApiEnum for $preference {
            fn to_api_value(&self) -> i8 {
                match self {
                    Self::OpenToAll => -1,
                    $(Self::$variant => $value,)*
                }
            }

            fn from_string(s: &str) -> Option<Self> {
                match s {
                    "OpenToAll" => Some(Self::OpenToAll),
                    $(stringify!($variant) => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Text,
    Media,
    Audio,
    Video,
    Voice,
    Poll,
}

impl_profile_preference_enum! {
    ChildrenStatusValue,
    ChildrenStatusProfile,
    ChildrenStatusPreference {
        No = 1,
        Yes = 2,
    }
}

impl_profile_preference_enum! {
    DatingIntentionValue,
    DatingIntentionProfile,
    DatingIntentionPreference {
        LifePartner = 1,
        LongTermRelationship = 2,
        LongTermOpenToShort = 3,
        ShortTermOpenToLong = 4,
        ShortTermRelationship = 5,
        FiguringOutTheirDatingGoals = 6,
    }
}

impl_profile_preference_enum! {
    DrinkingStatusValue,
    DrinkingStatusProfile,
    DrinkingStatusPreference {
        No = 1,
        Yes = 2,
        Sometimes = 3,
    }
}

impl_profile_preference_enum! {
    SmokingStatusValue,
    SmokingStatusProfile,
    SmokingStatusPreference {
        No = 1,
        Yes = 2,
        Sometimes = 3,
    }
}

impl_profile_preference_enum! {
    MarijuanaStatusValue,
    MarijuanaStatusProfile,
    MarijuanaStatusPreference {
        No = 1,
        Yes = 2,
        Sometimes = 3,
        NoPreference = 4,
    }
}

impl_profile_preference_enum! {
    DrugStatusValue,
    DrugStatusProfile,
    DrugStatusPreference {
        No = 1,
        Yes = 2,
        Sometimes = 3,
    }
}

impl_numeric_string_enum! {
    GenderEnum {
        Man = 0,
        Woman = 1,
        NonBinary = 3,
    }
}

impl_numeric_string_enum! {
    GenderPreferences {
        Men = 0,
        Women = 1,
        Everyone = 2,
    }
}

impl_profile_preference_enum! {
    EthnicityValue,
    EthnicityProfile,
    EthnicityPreference {
        AmericanIndian = 1,
        BlackAfrican = 2,
        EastAsian = 3,
        Hispanic = 4,
        MiddleEastern = 5,
        PacificIslander = 6,
        SouthAsian = 7,
        White = 8,
        Other = 9,
    }
}

impl_profile_preference_enum! {
    ReligionValue,
    ReligionProfile,
    ReligionPreference {
        Spiritual = 1,
        Catholic = 2,
        Christian = 3,
        Hindu = 4,
        Jewish = 5,
        Muslim = 6,
        Buddhist = 7,
        Agnostic = 8,
        Atheist = 9,
        Other = 10,
        Sikh = 11,
    }
}

impl_profile_preference_enum! {
    PoliticsValue,
    PoliticsProfile,
    PoliticsPreference {
        Liberal = 1,
        Moderate = 2,
        Conservative = 3,
        NotPolitical = 4,
        Other = 5,
    }
}

impl_profile_preference_enum! {
    EducationAttainedValue,
    EducationAttainedProfile,
    EducationAttainedPreference {
        HighSchool = 1,
        TradeSchool = 2,
        InCollege = 3,
        Undergraduate = 4,
        InGradSchool = 5,
        Graduate = 6,
    }
}

impl_profile_preference_enum! {
    RelationshipTypeValue,
    RelationshipTypeProfile,
    RelationshipTypePreference {
        Monogamy = 1,
        EthicalNonMonogamy = 2,
        FiguringOutTheirRelationshipType = 3,
    }
}

// QuestionId is actually a String UUID in the API, not an enum
pub type QuestionId = String;

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LanguageEnum {
    En,
    Es,
    Fr,
    De,
    It,
    Pt,
    Ru,
    Zh,
    Ja,
    Ko,
    Ar,
    Hi,
    Other,
}
