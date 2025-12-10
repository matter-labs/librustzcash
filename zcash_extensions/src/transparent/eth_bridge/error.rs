use std::fmt;

/// Errors that may be produced during parsing and verification of demo preconditions and
/// witnesses.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Parse error indicating that the payload of the condition or the witness was
    /// not correct.
    IllegalPayloadLength(usize),
    /// Verification error indicating that the specified mode was not recognized by
    /// the extension.
    ModeInvalid(u32),
    /// Verification error indicating that the transaction provided in the verification
    /// context was missing required TZE inputs or outputs.
    NonTzeTxn,
    /// Verification error indicating that the mode requested by the witness value did not
    /// conform to that of the precondition under inspection.
    ModeMismatch,
    /// Verification error indicating that the transaction didn't expect to have non-TZE bundles.
    NonTzeBundle,
    /// Verification error indicating that the transaction provided in the verification
    /// context had incorrect amount of TZE inputs.
    IncorrectTzeInputsAmount { expected: usize, provided: usize },
    /// Verification error indicating that the transaction provided in the verification
    /// context had incorrect amount of TZE outputs.
    IncorrectTzeOutputsAmount { expected: usize, provided: usize },
    /// Verification error indicating that the transaction provided in the verification
    /// context had incorrect amount of transparent outputs.
    IncorrectTransparentOutputsAmount { expected: usize, provided: usize },
    /// Verification error indicating that the output TZE has unexpected precondition.
    UnexpectedOutputPrecondition,
    /// Verification error indicating that the input TZE has unexpected witness.
    UnexpectedInputWitness,
    /// Verification error indicating that STF properties were not met.
    InvalidStfUpdate(String),
}

impl fmt::Display for Error {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        match self {
            Error::IllegalPayloadLength(sz) => write!(f, "Illegal payload length for demo: {}", sz),
            Error::ModeInvalid(m) => write!(f, "Invalid TZE mode for demo program: {}", m),
            Error::NonTzeTxn => write!(f, "Transaction has non-TZE inputs."),
            Error::ModeMismatch => write!(f, "Extension operation mode mismatch."),
            Error::NonTzeBundle => write!(f, "Non-TZE bundles detected in transaction"),
            Error::IncorrectTzeInputsAmount { expected, provided } => write!(
                f,
                "Incorrect TZE inputs amount, expected {expected}, provided {provided}"
            ),
            Error::IncorrectTzeOutputsAmount { expected, provided } => write!(
                f,
                "Incorrect TZE outputs amount, expected {expected}, provided {provided}"
            ),
            Error::IncorrectTransparentOutputsAmount { expected, provided } => write!(
                f,
                "Incorrect transparent outputs amount, expected {expected}, provided {provided}"
            ),
            Error::UnexpectedOutputPrecondition => {
                write!(f, "Unexpected encoding for output precondition")
            }
            Error::UnexpectedInputWitness => {
                write!(f, "Unexpected encoding for input witness")
            }
            Error::InvalidStfUpdate(msg) => write!(f, "Invalid STF update: {msg}"),
        }
    }
}
