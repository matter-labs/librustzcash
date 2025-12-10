use zcash_primitives::extensions::transparent::Extension;

use crate::transparent::eth_bridge::{
    context::Context,
    error::Error,
    modes::{create, deposit, stf},
    precondition::Precondition,
    witness::Witness,
};

pub struct Program;

impl<C: Context> Extension<C> for Program {
    type Precondition = Precondition;
    type Witness = Witness;
    type Error = Error;

    /// Runs the program against the given precondition, witness, and context.
    ///
    /// At this point the precondition and witness have been parsed and validated
    /// non-contextually, and are guaranteed to both be for this program. All subsequent
    /// validation is this function's responsibility.
    fn verify_inner(
        &self,
        precondition: &Precondition,
        witness: &Witness,
        context: &C,
    ) -> Result<(), Error> {
        // This match statement is selecting the mode that the program is operating in,
        // based on the enums defined in the parser.
        match (precondition, witness) {
            (Precondition::Create(p), Witness::Create(w)) => create::verify(p, w, context),
            (Precondition::Deposit(p), Witness::Deposit(w)) => deposit::verify(p, w, context),
            (Precondition::Stf(p), Witness::Stf(w)) => stf::verify(p, w, context),
            _ => Err(Error::ModeMismatch),
        }
    }
}
