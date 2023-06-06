/// Rperesents the error of the read_input functions
#[derive(Debug)]
pub enum ReadInputError {
    TimeToReboot,
    // TODO: fix this 
    GenericError(String),
    /// The message does not be process by this rollup
    NotATzwitterMessage,
    /// There is no more messages
    EndOfInbox,
    /// There is an error in the bytes to string deserialization
    FromUtf8Error(std::string::FromUtf8Error),
    /// There is an error in the string to Message deserialization
    SerdeJson(serde_json_wasm::de::Error),
    /// There is an error runtime
    Runtime(tezos_smart_rollup::host::RuntimeError),
}

/// Represents all the error of the kernel
///
#[derive(Debug)]
pub enum Error {
    FromUtf8(std::string::FromUtf8Error),
    Runtime(tezos_smart_rollup::host::RuntimeError),
    Ed25519Compact(ed25519_compact::Error),
    InvalidSignature,
    InvalidNonce,
    PathError(tezos_smart_rollup::storage::path::PathError),
    StateDeserializarion,
    BinError(tezos_data_encoding::enc::BinError),
    EntrypointError(tezos_smart_rollup::types::EntrypointError),
    GenericError(String) 
}

impl ToString for Error {
    fn to_string(&self) -> String {
        let err = match self {
            Error::FromUtf8(_) => "Cannot convert bytes to string",
            Error::Runtime(_) => "Runtime error, caused by host function",
            Error::Ed25519Compact(_) => "Cannot deserialize Ed25519",
            Error::InvalidSignature => "Invalid signature",
            Error::InvalidNonce => "Invalid nonce",
            Error::PathError(_) => "Invalid path",
            Error::StateDeserializarion => "State deserialization",
            Error::BinError(_) => "Cannot serialize michelson to binary",
            Error::EntrypointError(_) => "Not a correct entrypoint",
           Error::GenericError(_)  => "TODO: fix generic error message",
        };
        err.to_string()
    }
}

macro_rules! register_error {
    ($name:ident, $error:ty) => {
        impl From<$error> for Error {
            fn from(data: $error) -> Self {
                Error::$name(data)
            }
        }
    };
}

register_error!(FromUtf8, std::string::FromUtf8Error);
register_error!(Ed25519Compact, ed25519_compact::Error);
register_error!(PathError, tezos_smart_rollup::storage::path::PathError);
register_error!(Runtime, tezos_smart_rollup::host::RuntimeError);
register_error!(BinError, tezos_data_encoding::enc::BinError);
register_error!(EntrypointError, tezos_smart_rollup::types::EntrypointError);

pub type Result<A> = std::result::Result<A, Error>;
