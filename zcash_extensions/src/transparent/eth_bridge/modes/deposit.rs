//! Deposit mode is meant to lock funds into the STF.
//! The expectation is that only the STF with a matching identifier
//! can consume the UTXO created in this mode.

use zcash_primitives::extensions::transparent::FromPayload as _;

use crate::transparent::eth_bridge::Witness as WitnessEnum;

use crate::transparent::eth_bridge::{Context, Error};

pub const MODE: u32 = 2;

#[derive(Debug, PartialEq, Eq)]
pub struct Precondition {
    pub stf_identifier: [u8; 32],
    pub to: [u8; 20], // An Ethereum address
}

impl Precondition {
    pub fn new(stf_identifier: [u8; 32], to: [u8; 20]) -> Self {
        Self { stf_identifier, to }
    }

    pub fn to_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(52);
        payload.extend_from_slice(&self.stf_identifier);
        payload.extend_from_slice(&self.to);
        payload
    }
}

impl TryFrom<&[u8]> for Precondition {
    type Error = Error;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if payload.len() != 52 {
            return Err(Error::IllegalPayloadLength(payload.len()));
        }
        let stf_identifier = payload[0..32].try_into().unwrap();
        let to = payload[32..52].try_into().unwrap();
        Ok(Precondition::new(stf_identifier, to))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Witness; // TODO: Should we have anything here?

impl TryFrom<&[u8]> for Witness {
    type Error = Error;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if !payload.is_empty() {
            return Err(Error::IllegalPayloadLength(payload.len()));
        }
        Ok(Witness)
    }
}

impl Witness {
    pub fn to_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

pub fn verify(
    precondition: &Precondition,
    _witness: &Witness,
    context: &impl Context,
) -> Result<(), Error> {
    let tze_inputs = context.tx_tze_inputs();
    // We should have at least 2 TZE inputs: one STF input and at least one deposit input.
    if tze_inputs.len() < 2 {
        return Err(Error::IncorrectTzeInputsAmount {
            expected: 2,
            provided: tze_inputs.len(),
        });
    }

    let (_deposits_in, stf_in): (Vec<_>, Vec<_>) = tze_inputs
        .iter()
        .partition(|input| input.witness.mode == MODE);
    // Attempt to consume the deposit without STF update.
    if stf_in.len() != 1 {
        return Err(Error::InvalidStfUpdate(
            "expected exactly one STF TZE input".to_string(),
        ));
    }
    let stf_in = stf_in.first().expect("len == 1 enforced above");

    let Ok(WitnessEnum::Stf(stf_witness)) =
        WitnessEnum::from_payload(stf_in.witness.mode, &stf_in.witness.payload.0)
    else {
        return Err(Error::UnexpectedInputWitness);
    };

    if stf_witness.stf_identifier != precondition.stf_identifier {
        return Err(Error::InvalidStfUpdate(
            "STF identifier mismatch".to_string(),
        ));
    }

    Ok(())
}
