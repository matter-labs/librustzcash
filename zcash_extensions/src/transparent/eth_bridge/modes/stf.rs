//! The STF mode is meant to progress the bridge state,
//! processing deposits and withdrawals.
//!
//! Deposits are represented by UTXO in the Deposit mode,
//! while withdrawals are represented by transparent outputs.
//!
//! Both are enforeced through the witness, with the expectation
//! that hashes of the deposits and withdrawals are included into
//! the public input for ZK proof verification.

use std::collections::{HashMap, HashSet};

use crate::transparent::eth_bridge::Precondition as PreconditionEnum;

use transparent::address::TransparentAddress;
use zcash_primitives::extensions::transparent::FromPayload as _;
use zcash_protocol::value::Zatoshis;

use crate::transparent::eth_bridge::{Context, Error};

pub const MODE: u32 = 1;

#[derive(Debug, PartialEq, Eq)]
pub struct Precondition {
    pub root_hash: [u8; 32],
    pub stf_identifier: [u8; 32],
}

impl Precondition {
    pub fn new(stf_identifier: [u8; 32], root_hash: [u8; 32]) -> Self {
        Precondition {
            root_hash,
            stf_identifier,
        }
    }

    pub fn to_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(64);
        payload.extend_from_slice(&self.stf_identifier);
        payload.extend_from_slice(&self.root_hash);
        payload
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessedDeposit {
    pub to: [u8; 20], // An Ethereum address
    pub amount: Zatoshis,
}

impl ProcessedDeposit {
    pub fn to_payload(&self, payload: &mut Vec<u8>) {
        payload.extend_from_slice(&self.to);
        payload.extend_from_slice(&self.amount.into_u64().to_le_bytes());
    }
}

impl TryFrom<&[u8]> for ProcessedDeposit {
    type Error = Error;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if payload.len() != 28 {
            return Err(Error::IllegalPayloadLength(payload.len()));
        }
        let to = payload[0..20].try_into().unwrap();
        let amount = Zatoshis::from_u64_le_bytes(
            payload[20..28]
                .try_into()
                .expect("slice with correct length"),
        )
        .map_err(|_| Error::UnexpectedInputWitness)?;
        Ok(ProcessedDeposit { to, amount })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessedWithdrawal {
    pub pubkey_hash: [u8; 20], // A Zcash transparent address
    pub amount: Zatoshis,
}

impl TryFrom<&[u8]> for ProcessedWithdrawal {
    type Error = Error;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if payload.len() != 28 {
            return Err(Error::IllegalPayloadLength(payload.len()));
        }
        let pubkey_hash = payload[0..20].try_into().unwrap();
        let amount = Zatoshis::from_u64_le_bytes(
            payload[20..28]
                .try_into()
                .expect("slice with correct length"),
        )
        .map_err(|_| Error::UnexpectedInputWitness)?;
        Ok(ProcessedWithdrawal {
            pubkey_hash,
            amount,
        })
    }
}

impl ProcessedWithdrawal {
    pub fn to_payload(&self, payload: &mut Vec<u8>) {
        payload.extend_from_slice(&self.pubkey_hash);
        payload.extend_from_slice(&self.amount.into_u64().to_le_bytes());
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Witness {
    pub new_root_hash: [u8; 32],
    pub stf_identifier: [u8; 32],
    pub processed_deposits: Vec<ProcessedDeposit>,
    pub processed_withdrawals: Vec<ProcessedWithdrawal>,
    // TODO: zk proof & system params.
}

impl TryFrom<&[u8]> for Witness {
    type Error = Error;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        if payload.len() < 72 {
            return Err(Error::IllegalPayloadLength(payload.len()));
        }
        let stf_identifier = payload[0..32].try_into().unwrap();
        let new_root_hash = payload[32..64].try_into().unwrap();

        let n_deposits = u32::from_le_bytes(
            payload[64..68]
                .try_into()
                .expect("slice with correct length"),
        ) as usize;
        let n_withdrawals = u32::from_le_bytes(
            payload[68..72]
                .try_into()
                .expect("slice with correct length"),
        ) as usize;

        if payload.len() != 72 + n_deposits * 28 + n_withdrawals * 28 {
            return Err(Error::IllegalPayloadLength(payload.len()));
        }

        let mut processed_deposits = Vec::new();
        let mut processed_withdrawals = Vec::new();

        for i in 0..n_deposits {
            let start = 72 + i * 28;
            let end = start + 28;
            let deposit = ProcessedDeposit::try_from(&payload[start..end])?;
            processed_deposits.push(deposit);
        }

        for i in 0..n_withdrawals {
            let start = 72 + n_deposits * 28 + i * 28;
            let end = start + 28;
            let withdrawal = ProcessedWithdrawal::try_from(&payload[start..end])?;
            processed_withdrawals.push(withdrawal);
        }

        Ok(Witness {
            new_root_hash,
            stf_identifier,
            processed_deposits,
            processed_withdrawals,
        })
    }
}

impl Witness {
    pub fn to_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(64);
        payload.extend_from_slice(&self.stf_identifier);
        payload.extend_from_slice(&self.new_root_hash);
        let n_deposits = self.processed_deposits.len() as u32;
        payload.extend_from_slice(&n_deposits.to_le_bytes());
        let n_withdrawals = self.processed_withdrawals.len() as u32;
        payload.extend_from_slice(&n_withdrawals.to_le_bytes());

        for deposit in &self.processed_deposits {
            deposit.to_payload(&mut payload);
        }
        for withdrawal in &self.processed_withdrawals {
            withdrawal.to_payload(&mut payload);
        }

        payload
    }
}

pub fn verify(
    precondition: &Precondition,
    witness: &Witness,
    context: &impl Context,
) -> Result<(), Error> {
    // TODO: verify that the input for this tx was originated _also_ from either `Create` or `Stf` transaction,
    // e.g. it was not a coinbase `Stf` object.

    // TODO: Should we check that we have at most one transparent input?

    // We must have N deposits + 1 STF TZE inputs.
    let expected_inputs_amount = witness.processed_deposits.len() + 1;
    let tze_inputs = context.tx_tze_inputs();
    if tze_inputs.len() != expected_inputs_amount {
        return Err(Error::IncorrectTzeInputsAmount {
            expected: expected_inputs_amount,
            provided: tze_inputs.len(),
        });
    }

    // We must have 1 STF TZE outputs.
    let tze_outputs = context.tx_tze_outputs();
    if tze_outputs.len() != 1 {
        return Err(Error::IncorrectTzeOutputsAmount {
            expected: 1,
            provided: tze_outputs.len(),
        });
    }

    // We must have at most 1 tranpsarent output for fees, plus N withdrawals.
    let expected_outputs_amount = witness.processed_withdrawals.len();
    if context.tx_transparent_outputs().len() < expected_outputs_amount {
        return Err(Error::IncorrectTransparentOutputsAmount {
            expected: expected_outputs_amount,
            provided: context.tx_transparent_outputs().len(),
        });
    }

    if precondition.root_hash != witness.new_root_hash {
        return Err(Error::InvalidStfUpdate("Root hash mismatch".to_string())); // TODO: Better errors & granularity.
    }
    if precondition.stf_identifier != witness.stf_identifier {
        return Err(Error::InvalidStfUpdate(
            "STF identifier mismatch".to_string(),
        ));
    }

    let (stf_in, deposits_in): (Vec<_>, Vec<_>) = tze_inputs
        .iter()
        .partition(|input| input.witness.mode == MODE);
    if stf_in.len() != 1 || deposits_in.len() != witness.processed_deposits.len() {
        return Err(Error::InvalidStfUpdate(
            "Incorrect number of STF or deposit inputs".to_string(),
        ));
    }

    // Match tze inputs to processed deposits.
    let deposit_by_address = {
        let mut deposit_by_address = HashMap::new();
        for deposit in &witness.processed_deposits {
            deposit_by_address.insert(deposit.to, deposit.amount); // TODO: must be a `Vec`, since multiple deposits can go to the same address.
        }
        deposit_by_address
    };

    for deposit_input in deposits_in {
        // TODO: fetch precondition for deposits & verify them.
        // let deposit_precondition = todo!();

        // let Ok(PreconditionEnum::Deposit(p)) =
        //     PreconditionEnum::from_payload(deposit_input.witness.mode, &deposit_input.witness.payload.0)
        // else {
        //     return Err(Error::UnexpectedInputWitness);
        // };

        // let _expected_deposit = deposit_by_address.get(&p.to).ok_or_else(|| {
        //     Error::InvalidStfUpdate(format!("Deposit for address {:x?} is not present", p.to))
        // })?;

        // TODO: Check amounts match.
    }

    let stf_out = tze_outputs.first().expect("len == 1 enforced above");

    let Ok(PreconditionEnum::Stf(p)) =
        PreconditionEnum::from_payload(stf_out.precondition.mode, &stf_out.precondition.payload)
    else {
        return Err(Error::UnexpectedOutputPrecondition);
    };

    if p.root_hash != witness.new_root_hash {
        return Err(Error::InvalidStfUpdate("Root hash mismatch".to_string()));
    }

    let withdrawal_by_pubkey_hash = {
        let mut map = HashMap::new();
        for withdrawal in &witness.processed_withdrawals {
            map.insert(withdrawal.pubkey_hash, withdrawal); // TODO: must be a `Vec`, since multiple withdrawals can go to the same address.
        }
        map
    };
    let mut not_covered_withdrawals: HashSet<[u8; 20]> = witness
        .processed_withdrawals
        .iter()
        .map(|w| w.pubkey_hash)
        .collect();

    for t_out in context.tx_transparent_outputs() {
        let Some(TransparentAddress::PublicKeyHash(recipient)) = t_out.recipient_address() else {
            // Withdrawal must be a pubkey hash.
            continue;
        };
        let Some(withdrawal) = withdrawal_by_pubkey_hash.get(&recipient) else {
            // Non-withdrawal transparent output (e.g., fee).
            continue;
        };

        if t_out.value() != withdrawal.amount {
            return Err(Error::InvalidStfUpdate(
                "Transparent output value does not match withdrawal amount".to_string(),
            ));
        }
        not_covered_withdrawals.remove(&recipient);
    }

    if !not_covered_withdrawals.is_empty() {
        return Err(Error::InvalidStfUpdate(
            "Some withdrawals were not covered by transparent outputs".to_string(),
        ));
    }

    Ok(())
}
