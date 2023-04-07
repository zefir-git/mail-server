use std::fmt::Display;

use crate::{
    error::method::MethodError,
    parser::{json::Parser, Error, JsonObjectParser, Token},
    types::{id::Id, pointer::JSONPointer},
};

use super::method::MethodName;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ResultReference {
    #[serde(rename = "resultOf")]
    pub result_of: String,
    pub name: MethodName,
    pub path: JSONPointer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaybeReference<V, R> {
    Value(V),
    Reference(R),
}

impl JsonObjectParser for ResultReference {
    fn parse(parser: &mut Parser) -> crate::parser::Result<Self>
    where
        Self: Sized,
    {
        let mut result_of = None;
        let mut name = None;
        let mut path = None;

        parser
            .next_token::<String>()?
            .assert_jmap(Token::DictStart)?;

        while {
            match parser.next_dict_key::<u64>()? {
                0x664f_746c_7573_6572 => {
                    result_of = Some(parser.next_token::<String>()?.unwrap_string("resultOf")?);
                }
                0x656d_616e => {
                    name = Some(parser.next_token::<MethodName>()?.unwrap_string("name")?);
                }
                0x6874_6170 => {
                    path = Some(parser.next_token::<JSONPointer>()?.unwrap_string("path")?);
                }
                _ => {
                    parser.skip_token(parser.depth_array, parser.depth_dict)?;
                }
            }

            !parser.is_dict_end()?
        } {}

        if let (Some(result_of), Some(name), Some(path)) = (result_of, name, path) {
            Ok(Self {
                result_of,
                name,
                path,
            })
        } else {
            Err(Error::Method(MethodError::InvalidResultReference(
                "Missing required fields".into(),
            )))
        }
    }
}

impl Display for ResultReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ resultOf: {}, name: {}, path: {} }}",
            self.result_of, self.name, self.path
        )
    }
}

impl<V: Display, R: Display> Display for MaybeReference<V, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybeReference::Value(id) => write!(f, "{}", id),
            MaybeReference::Reference(str) => write!(f, "#{}", str),
        }
    }
}

// MaybeReference de/serialization
impl serde::Serialize for MaybeReference<Id, String> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            MaybeReference::Value(id) => id.serialize(serializer),
            MaybeReference::Reference(str) => serializer.serialize_str(&format!("#{}", str)),
        }
    }
}