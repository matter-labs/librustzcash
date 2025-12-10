use transparent::bundle::TxOut;
use zcash_primitives::{
    extensions::transparent::AuthData,
    transaction::components::{TzeIn, TzeOut},
};

pub trait Context {
    fn is_tze_only(&self) -> bool;

    fn tx_tze_inputs(&self) -> &[TzeIn<AuthData>];

    fn tx_tze_outputs(&self) -> &[TzeOut];

    fn tx_transparent_outputs(&self) -> &[TxOut];
}
