use zcash_primitives::extensions::transparent::{FromPayload, ToPayload};

use crate::transparent::eth_bridge::{
    error::Error,
    modes::{create, deposit, stf},
};

/// The witness type for the `eth_bridge` extension.
#[derive(Debug, PartialEq, Eq)]
pub enum Witness {
    Create(create::Witness),
    Deposit(deposit::Witness),
    Stf(stf::Witness),
}

impl Witness {
    pub fn create() -> Self {
        Self::Create(create::Witness)
    }

    pub fn deposit() -> Self {
        Self::Deposit(deposit::Witness)
    }
}

impl TryFrom<(u32, Witness)> for Witness {
    type Error = Error;

    fn try_from(from: (u32, Self)) -> Result<Self, Self::Error> {
        match from {
            (create::MODE, w @ Witness::Create(_)) => Ok(w),
            (stf::MODE, w @ Witness::Stf(_)) => Ok(w),
            (deposit::MODE, w @ Witness::Deposit(_)) => Ok(w),
            _ => Err(Error::ModeInvalid(from.0)),
        }
    }
}

impl FromPayload for Witness {
    type Error = Error;

    fn from_payload(mode: u32, payload: &[u8]) -> Result<Self, Self::Error> {
        match mode {
            create::MODE => payload
                .try_into()
                .map_err(|_| Error::IllegalPayloadLength(payload.len()))
                .map(Witness::Create),
            stf::MODE => payload
                .try_into()
                .map_err(|_| Error::IllegalPayloadLength(payload.len()))
                .map(Witness::Stf),
            deposit::MODE => payload
                .try_into()
                .map_err(|_| Error::IllegalPayloadLength(payload.len()))
                .map(Witness::Deposit),
            _ => Err(Error::ModeInvalid(mode)),
        }
    }
}

impl ToPayload for Witness {
    fn to_payload(&self) -> (u32, Vec<u8>) {
        match self {
            Witness::Create(w) => (create::MODE, w.to_payload()),
            Witness::Stf(w) => (stf::MODE, w.to_payload()),
            Witness::Deposit(w) => (deposit::MODE, w.to_payload()),
        }
    }
}
