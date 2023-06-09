use crate::error::*;
use crate::hash::Blake2b;
use crate::public_key::PublicKey;
use serde::{Deserialize, Serialize};
use tezos_crypto_rs::hash::Ed25519Signature;

#[derive(Deserialize, Serialize, Debug)]
pub enum Signature {
    Ed25519(Ed25519Signature),
}

impl Signature {
    pub fn verify(&self, public_key: &PublicKey, message: &[u8]) -> Result<()> {
        match (self, public_key) {
            (Signature::Ed25519(sig), PublicKey::Ed25519(pkey)) => {
                // TODO: There should be another way to do it
                // TODO: remove the unwrap
                let data = Blake2b::from(message);
                let data = data.as_ref();
                let signature =
                    ed25519_compact::Signature::from_slice(sig.as_ref()).map_err(Error::from)?;
                let pkey =
                    ed25519_compact::PublicKey::from_slice(pkey.as_ref()).map_err(Error::from)?;

                pkey.verify(data, &signature)
                    .map_err(|_| Error::InvalidSignature)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tezos_crypto_rs::hash::Ed25519Signature;

    use super::Signature;
    use crate::public_key::PublicKey;

    impl Signature {
        pub fn to_b58(&self) -> String {
            match self {
                Signature::Ed25519(sig) => sig.to_base58_check(),
            }
        }

        pub fn from_b58(data: &str) -> std::result::Result<Self, &'static str> {
            let ed25519 = Ed25519Signature::from_base58_check(data).ok();
            match ed25519 {
                Some(pkey) => Ok(Signature::Ed25519(pkey)),
                None => Err("Cannot decode b58"),
            }
        }
    }

    #[test]
    fn test_ed25519_signature_deserialization() {
        let signature = "edsigu1mRCtZquLvspcxaYXVZdsKKSqHnXevnrmh1T63Dq1Rr8M1giVLvapiDFK6TQCEyY6xytdGnKgZyVSHDVnub7puy54bD1y";
        let res = Signature::from_b58(signature);
        assert!(res.is_ok());
    }

    #[test]
    fn test_ed25519_signature_serialization() {
        let sig = "edsigu1mRCtZquLvspcxaYXVZdsKKSqHnXevnrmh1T63Dq1Rr8M1giVLvapiDFK6TQCEyY6xytdGnKgZyVSHDVnub7puy54bD1y";
        let serialized = Signature::from_b58(sig).unwrap().to_b58();
        assert_eq!(sig, &serialized)
    }

    #[test]
    fn test_ed25519_signature_verification() {
        let signature = Signature::from_b58("edsigu1mRCtZquLvspcxaYXVZdsKKSqHnXevnrmh1T63Dq1Rr8M1giVLvapiDFK6TQCEyY6xytdGnKgZyVSHDVnub7puy54bD1y").unwrap();
        let pkey =
            PublicKey::from_b58("edpkuDMUm7Y53wp4gxeLBXuiAhXZrLn8XB1R83ksvvesH8Lp8bmCfK").unwrap();
        let data = [
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
        ]
        .as_slice();

        let verification = signature.verify(&pkey, data);
        assert!(verification.is_ok());
    }
}
