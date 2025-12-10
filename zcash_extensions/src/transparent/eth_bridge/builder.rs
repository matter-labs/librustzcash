use std::ops::{Deref, DerefMut};

use zcash_primitives::{
    extensions::transparent::{ExtensionTxBuilder, FromPayload},
    transaction::components::tze::{OutPoint, TzeOut},
};
use zcash_protocol::value::Zatoshis;

use crate::transparent::eth_bridge::{Precondition, Witness, error::Error, modes};

/// Wrapper for [`zcash_primitives::transaction::builder::Builder`] that simplifies
/// constructing transactions that utilize the features of the `eth_bridge` extension.
pub struct EthBridgeTzeBuilder<B> {
    /// The wrapped transaction builder.
    pub txn_builder: B,

    /// The assigned identifier for this extension. This is necessary as the author
    /// of the `eth_bridge` extension will not know ahead of time what identifier will be
    /// assigned to it at the time of inclusion in the Zcash consensus rules.
    pub extension_id: u32,
}

impl<B> Deref for EthBridgeTzeBuilder<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.txn_builder
    }
}

impl<B> DerefMut for EthBridgeTzeBuilder<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.txn_builder
    }
}

/// Errors that can occur in construction of transactions using `EthBridgeTze`.
#[derive(Debug)]
pub enum EthBridgeTzeBuildError<E> {
    /// Wrapper for errors returned from the underlying `Builder`
    BaseBuilderError(E),
    ParseError(Error),
}

impl<E: std::fmt::Display> std::fmt::Display for EthBridgeTzeBuildError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EthBridgeTzeBuildError::BaseBuilderError(e) => write!(f, "Builder error: {}", e),
            EthBridgeTzeBuildError::ParseError(error) => write!(f, "Parse error: {}", error),
        }
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for EthBridgeTzeBuildError<E> {}

/// Convenience methods for use with [`zcash_primitives::transaction::builder::Builder`]
/// for constructing transactions that utilize the features of the `eth_bridge` extension.
impl<'a, B: ExtensionTxBuilder<'a>> EthBridgeTzeBuilder<B> {
    /// Add a channel-opening precondition to the outputs of the transaction under
    /// construction.
    pub fn add_create_output(
        &mut self,
        value: Zatoshis,
        stf_identifier: [u8; 32],
        root_hash: [u8; 32],
    ) -> Result<(), EthBridgeTzeBuildError<B::BuildError>> {
        // Call through to the generic builder.
        self.txn_builder
            .add_tze_output(
                self.extension_id,
                value,
                &Precondition::create(stf_identifier, root_hash),
            )
            .map_err(EthBridgeTzeBuildError::BaseBuilderError)
    }

    /// Consume the create TZE, allowing migration to the STF mode.
    pub fn add_create_input(
        &mut self,
        prevout: (OutPoint, TzeOut),
    ) -> Result<(), EthBridgeTzeBuildError<B::BuildError>> {
        match Precondition::from_payload(
            prevout.1.precondition.mode,
            &prevout.1.precondition.payload,
        ) {
            Err(parse_error) => Err(EthBridgeTzeBuildError::ParseError(parse_error)),
            Ok(Precondition::Create(_)) => self
                .txn_builder
                .add_tze_input(self.extension_id, modes::create::MODE, prevout, |_| {
                    Ok(Witness::create())
                })
                .map_err(EthBridgeTzeBuildError::BaseBuilderError),
            Ok(_) => Err(EthBridgeTzeBuildError::ParseError(Error::ModeInvalid(
                prevout.1.precondition.mode,
            ))),
        }
    }

    /// Add a deposit precondition to the outputs of the transaction under
    /// construction.
    pub fn add_deposit_output(
        &mut self,
        value: Zatoshis,
        stf_identifier: [u8; 32],
        to: [u8; 20],
    ) -> Result<(), EthBridgeTzeBuildError<B::BuildError>> {
        self.txn_builder
            .add_tze_output(
                self.extension_id,
                value,
                &Precondition::deposit(stf_identifier, to),
            )
            .map_err(EthBridgeTzeBuildError::BaseBuilderError)
    }

    /// Consume the deposit.
    pub fn add_deposit_input(
        &mut self,
        prevout: (OutPoint, TzeOut),
    ) -> Result<(), EthBridgeTzeBuildError<B::BuildError>> {
        match Precondition::from_payload(
            prevout.1.precondition.mode,
            &prevout.1.precondition.payload,
        ) {
            Err(parse_error) => Err(EthBridgeTzeBuildError::ParseError(parse_error)),
            Ok(Precondition::Deposit(_)) => self
                .txn_builder
                .add_tze_input(self.extension_id, modes::deposit::MODE, prevout, |_| {
                    Ok(Witness::deposit())
                })
                .map_err(EthBridgeTzeBuildError::BaseBuilderError),
            Ok(_) => Err(EthBridgeTzeBuildError::ParseError(Error::ModeInvalid(
                prevout.1.precondition.mode,
            ))),
        }
    }

    /// Add an STF precondition to the outputs of the transaction under
    /// construction.
    pub fn add_stf_output(
        &mut self,
        value: Zatoshis,
        stf_identifier: [u8; 32],
        root_hash: [u8; 32],
    ) -> Result<(), EthBridgeTzeBuildError<B::BuildError>> {
        self.txn_builder
            .add_tze_output(
                self.extension_id,
                value,
                &Precondition::stf(stf_identifier, root_hash),
            )
            .map_err(EthBridgeTzeBuildError::BaseBuilderError)
    }

    /// Adds an STF witness to the inputs of the transaction under construction.
    pub fn add_stf_input(
        &mut self,
        prevout: (OutPoint, TzeOut),
        stf_identifier: [u8; 32],
        root_hash: [u8; 32],
        processed_deposits: Vec<super::modes::stf::ProcessedDeposit>,
        processed_withdrawals: Vec<super::modes::stf::ProcessedWithdrawal>,
    ) -> Result<(), EthBridgeTzeBuildError<B::BuildError>> {
        let witness = super::modes::stf::Witness {
            stf_identifier,
            new_root_hash: root_hash,
            processed_deposits,
            processed_withdrawals,
        };

        match Precondition::from_payload(
            prevout.1.precondition.mode,
            &prevout.1.precondition.payload,
        ) {
            Err(parse_error) => Err(EthBridgeTzeBuildError::ParseError(parse_error)),
            Ok(Precondition::Stf(_)) => self
                .txn_builder
                .add_tze_input(self.extension_id, modes::stf::MODE, prevout, move |_| {
                    Ok(Witness::Stf(witness))
                })
                .map_err(EthBridgeTzeBuildError::BaseBuilderError),
            Ok(_) => Err(EthBridgeTzeBuildError::ParseError(Error::ModeInvalid(
                prevout.1.precondition.mode,
            ))),
        }
    }
}
