//! Create mode is meant to "claim" an STF identified by a unique identifier
//! The unique identifier is required, since multiple STFs can be instantiated,
//! and we need a measure to make sure that deposits can only be claimed
//! by the correct STF instance.

use zcash_primitives::extensions::transparent::FromPayload;

use crate::transparent::eth_bridge::Precondition as PreconditionEnum;
use crate::transparent::eth_bridge::{Context, Error};

pub const MODE: u32 = 0;

#[derive(Debug, PartialEq, Eq)]
pub struct Precondition {
    pub(crate) stf_identifier: [u8; 32], // TODO: should be a hash of signature for pre-defined message to make it unique enforced through witness?
    pub(crate) root_hash: [u8; 32],
}

impl Precondition {
    pub fn new(stf_identifier: [u8; 32], root_hash: [u8; 32]) -> Self {
        Precondition {
            stf_identifier,
            root_hash,
        }
    }

    // TODO: better serde logic.
    pub fn to_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(64);
        payload.extend_from_slice(&self.stf_identifier);
        payload.extend_from_slice(&self.root_hash);
        payload
    }
}

// TODO: better serde logic.
impl TryFrom<&[u8]> for Precondition {
    type Error = Error;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if payload.len() != 64 {
            return Err(Error::IllegalPayloadLength(payload.len()));
        }
        let stf_identifier = payload[0..32].try_into().unwrap();
        let root_hash = payload[32..64].try_into().unwrap();
        Ok(Precondition::new(stf_identifier, root_hash))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Witness;

impl Witness {
    pub fn to_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

// TODO: better serde logic.
impl TryFrom<&[u8]> for Witness {
    type Error = Error;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if !payload.is_empty() {
            return Err(Error::IllegalPayloadLength(payload.len()));
        }
        Ok(Witness {})
    }
}

pub fn verify(
    _precondition: &Precondition,
    _witness: &Witness,
    context: &impl Context,
) -> Result<(), Error> {
    // If this output is being consumed, we require that the chain is moved into
    // the `stf` state.
    // No other TZE inputs or TZE outputs are allowed in the transaction.

    // TODO: Should we check that we have at most one transparent input?

    let tze_inputs_count = context.tx_tze_inputs().len();
    if tze_inputs_count > 1 {
        return Err(Error::IncorrectTzeInputsAmount {
            expected: 1,
            provided: tze_inputs_count,
        });
    }
    let tze_outputs = context.tx_tze_outputs();
    if tze_outputs.len() != 1 {
        return Err(Error::IncorrectTzeOutputsAmount {
            expected: 1,
            provided: tze_outputs.len(),
        });
    }

    let output = tze_outputs.first().expect("len == 1 enforced above");
    let Ok(PreconditionEnum::Stf(p)) =
        PreconditionEnum::from_payload(output.precondition.mode, &output.precondition.payload)
    else {
        return Err(Error::UnexpectedOutputPrecondition);
    };

    Ok(())
}
