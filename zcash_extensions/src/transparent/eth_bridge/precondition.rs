use zcash_primitives::extensions::transparent::{FromPayload, ToPayload};

use crate::transparent::eth_bridge::error::Error;
use crate::transparent::eth_bridge::modes::{create, deposit, stf};

/// The precondition type for the eth_bridge extension.
#[derive(Debug, PartialEq, Eq)]
pub enum Precondition {
    Create(create::Precondition),
    Stf(stf::Precondition),
    Deposit(deposit::Precondition),
}

impl Precondition {
    pub fn create(stf_identifier: [u8; 32], root_hash: [u8; 32]) -> Self {
        Self::Create(create::Precondition::new(stf_identifier, root_hash))
    }

    pub fn deposit(stf_identifier: [u8; 32], to: [u8; 20]) -> Self {
        Self::Deposit(deposit::Precondition::new(stf_identifier, to))
    }

    pub fn stf(stf_identifier: [u8; 32], root_hash: [u8; 32]) -> Self {
        Self::Stf(stf::Precondition::new(stf_identifier, root_hash))
    }
}

impl TryFrom<(u32, Precondition)> for Precondition {
    type Error = Error;

    fn try_from(from: (u32, Self)) -> Result<Self, Self::Error> {
        match from {
            (create::MODE, Precondition::Create(p)) => Ok(Precondition::Create(p)),
            (stf::MODE, Precondition::Stf(p)) => Ok(Precondition::Stf(p)),
            (deposit::MODE, Precondition::Deposit(p)) => Ok(Precondition::Deposit(p)),
            _ => Err(Error::ModeInvalid(from.0)),
        }
    }
}

impl FromPayload for Precondition {
    type Error = Error;

    fn from_payload(mode: u32, payload: &[u8]) -> Result<Self, Self::Error> {
        match mode {
            create::MODE => payload
                .try_into()
                .map_err(|_| Error::IllegalPayloadLength(payload.len()))
                .map(Precondition::Create),
            stf::MODE => payload
                .try_into()
                .map_err(|_| Error::IllegalPayloadLength(payload.len()))
                .map(Precondition::Stf),
            deposit::MODE => payload
                .try_into()
                .map_err(|_| Error::IllegalPayloadLength(payload.len()))
                .map(Precondition::Deposit),
            _ => Err(Error::ModeInvalid(mode)),
        }
    }
}

impl ToPayload for Precondition {
    fn to_payload(&self) -> (u32, Vec<u8>) {
        match self {
            Precondition::Create(p) => (create::MODE, p.to_payload()),
            Precondition::Stf(p) => (stf::MODE, p.to_payload()),
            Precondition::Deposit(p) => (deposit::MODE, p.to_payload()),
        }
    }
}
