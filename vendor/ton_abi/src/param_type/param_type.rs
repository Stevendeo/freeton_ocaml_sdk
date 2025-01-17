/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

//! Function and event param types.

use std::fmt;
use Param;

use crate::AbiError;

use ton_types::{error, Result};

/// Function and event param types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamType {
    Unknown,
    /// uint<M>: unsigned integer type of M bits.
    Uint(usize),
    /// int<M>: signed integer type of M bits.
    Int(usize),
    /// bool: boolean value.
    Bool,
    /// Tuple: several values combined into tuple.
    Tuple(Vec<Param>),
    /// T[]: dynamic array of elements of the type T.
    Array(Box<ParamType>),
    /// T[k]: dynamic array of elements of the type T.
    FixedArray(Box<ParamType>, usize),
    /// cell - tree of cells
    Cell,
    /// hashmap - values dictionary
    Map(Box<ParamType>, Box<ParamType>),
    /// TON message address
    Address,
    /// byte array
    Bytes,
    /// fixed size byte array
    FixedBytes(usize),
    /// Nanograms
    Gram,
    /// Timestamp
    Time,
    /// Message expiration time
    Expire,
    /// Public key
    PublicKey
}

impl fmt::Display for ParamType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.type_signature())
    }
}

impl ParamType {
    /// Returns type signature according to ABI specification
    pub fn type_signature(&self) -> String {
        match self {
            ParamType::Unknown => format!("unknown"),
            ParamType::Uint(size) => format!("uint{}", size),
            ParamType::Int(size) => format!("int{}", size),
            ParamType::Bool => "bool".to_owned(),
            ParamType::Tuple(params) => {
                let mut signature = "".to_owned();
                for param in params {
                    signature += ",";
                    signature += &param.kind.type_signature();
                }
                signature.replace_range(..1, "(");
                signature + ")"
            },
            ParamType::Array(ref param_type) => format!("{}[]", param_type.type_signature()),
            ParamType::FixedArray(ref param_type, size) => 
                format!("{}[{}]", param_type.type_signature(), size),
            ParamType::Cell => "cell".to_owned(),
            ParamType::Map(key_type, value_type) => 
                format!("map({},{})", key_type.type_signature(), value_type.type_signature()),
            ParamType::Address => format!("address"),
            ParamType::Bytes => format!("bytes"),
            ParamType::FixedBytes(size) => format!("fixedbytes{}", size),
            ParamType::Gram => format!("gram"),
            ParamType::Time => format!("time"),
            ParamType::Expire => format!("expire"),
            ParamType::PublicKey => format!("pubkey"),
        }
    }

    pub fn set_components(&mut self, components: Vec<Param>) -> Result<()> {
        match self {
            ParamType::Tuple(params) => {
                if components.len() == 0 {
                    Err(error!(AbiError::EmptyComponents))
                } else {
                    Ok(*params = components)
                }
            } 
            ParamType::Array(array_type) => {
                array_type.set_components(components)
            }
            ParamType::FixedArray(array_type, _) => {
                array_type.set_components(components)
            }
            ParamType::Map(_, value_type) => {
                value_type.set_components(components)
            }
            _ => { 
                if components.len() != 0 {
                    Err(error!(AbiError::UnusedComponents))
                } else {
                    Ok(())
                }
            },
        }
    }

    /// Returns type bit_len for hashmap key
    pub fn bit_len(&self) -> usize {
        match self {
            ParamType::Uint(size) => *size,
            ParamType::Int(size) => *size,
            _ => 0
        }
    }

    /// Check if parameter type is supoorted in particular ABI version
    pub fn is_supported(&self, abi_version: u8) -> bool {
        match self {
            ParamType::Time | ParamType::Expire | ParamType::PublicKey => abi_version >= 2,
            _ => abi_version >= 1
        }
    }

    pub fn get_map_key_size(&self) -> Result<usize> {
        match self {
            ParamType::Int(size) | ParamType::Uint(size) => Ok(*size),
            ParamType::Address => Ok(crate::token::STD_ADDRESS_BIT_LENGTH),
            _ => Err(error!(AbiError::InvalidData { 
                msg: "Only integer and std address values can be map keys".to_owned() 
            }))
        }
    }
}
