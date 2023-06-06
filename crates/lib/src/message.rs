use crate::constants::MAGIC_BYTE;
use crate::hash::Blake2b;
use crate::nonce::Nonce;
use crate::public_key::PublicKey;
use crate::signature::Signature;
use serde::{Deserialize, Serialize};
use tezos_crypto_rs::hash::HashTrait;
use tezos_data_encoding::enc::BinWriter;
use tezos_smart_rollup::core_unsafe::PREIMAGE_HASH_SIZE;

#[derive(Deserialize, Serialize, Debug)]
pub struct PlacePixel {
    pub x: u32,
    pub y: u32,
    pub color: [u8; 3],
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Content {
    PlacePixel(PlacePixel),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Inner {
    nonce: Nonce,
    pub content: Content,
}

impl Inner {
    /// Returns the nonce of the inner
    pub fn nonce(&self) -> &Nonce {
        &self.nonce
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserMessage {
    pkey: PublicKey,
    signature: Signature,
    pub inner: Inner,
}

impl UserMessage {
    /// Returns the public key of the message
    pub fn public_key(&self) -> &PublicKey {
        &self.pkey
    }

    /// Returns the signature of the message
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// Returns the inner of the message
    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    /// Returns the hash of the message
    pub fn hash(&self) -> Blake2b {
        self.inner.hash()
    }

    pub fn new(skey: ed25519_compact::SecretKey, inner: Inner) -> Self {
        let data_to_hash = inner.hash();
        let data_to_hash = Blake2b::from(data_to_hash.as_ref());
        let signature = skey.sign(data_to_hash, None);
        let signature = signature.as_ref();
        let signature = tezos_crypto_rs::hash::Ed25519Signature::try_from_bytes(signature).unwrap();
        let signature = Signature::Ed25519(signature);
        let pkey = skey.public_key();
        let pkey = pkey.as_ref();
        let pkey = tezos_crypto_rs::hash::PublicKeyEd25519::try_from_bytes(pkey).unwrap();
        let pkey = PublicKey::Ed25519(pkey);
        UserMessage {
            pkey,
            signature,
            inner,
        }
    }
}

impl Inner {
    /// Hash of the message
    /// This hash is what the client should signed
    pub fn hash(&self) -> Blake2b {
        let json = serde_json_wasm::to_string(self).unwrap();
        println!("=========== hashing data: {}", json);
        let hash = Blake2b::from(json.as_bytes());
        println!("{}", hash.to_string());
        hash
    }
}

#[derive(Deserialize, Serialize)]
pub struct Message {
    pub signature: Signature,
    // FIXME: this is wrong. I should just figureo ut how to serizlie the 33 bytes.
    // I should also inlcude the magic byte 
    pub unprefixed_merkle_root: [u8; (PREIMAGE_HASH_SIZE - 1)],
}
impl Message {
    pub fn new(
        skey: ed25519_compact::SecretKey,
        unprefixed_merkle_root: [u8; (PREIMAGE_HASH_SIZE - 1)],
    ) -> Self {
        let data_to_sign = Blake2b::from(unprefixed_merkle_root.as_ref());
        // FIXME: signing this is wrong. See above comment.
        let signature = skey.sign(data_to_sign, None);
        let signature = signature.as_ref();
        let signature = tezos_crypto_rs::hash::Ed25519Signature::try_from_bytes(signature).unwrap();
        let signature = Signature::Ed25519(signature);
        Message {
            signature,
            unprefixed_merkle_root,
        }
    }
}

impl BinWriter for Message {
    fn bin_write(&self, output: &mut Vec<u8>) -> tezos_data_encoding::enc::BinResult {
        let bytes: Vec<u8> = serde_json_wasm::to_vec(&self).unwrap();
        output.extend_from_slice(&[MAGIC_BYTE]);
        output.extend_from_slice(&bytes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tezos_crypto_rs::hash::HashTrait;

    use super::{Content, Inner, PlacePixel};
    use crate::{hash::Blake2b, message::UserMessage, nonce::Nonce};

    #[test]
    fn expect() {
        let inner = Inner {
            nonce: Nonce(777),
            content: Content::PlacePixel(PlacePixel {
                x: 1,
                y: 2,
                color: [1, 2, 3],
            }),
        };

        let seed = ed25519_compact::Seed::new([
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ]);
        let ed25519_compact::KeyPair { pk, sk } = ed25519_compact::KeyPair::from_seed(seed);

        let sk_b58 = tezos_crypto_rs::hash::SecretKeyEd25519::try_from_bytes(sk.as_ref()).unwrap();
        let pk_b58 = tezos_crypto_rs::hash::PublicKeyEd25519::try_from_bytes(pk.as_ref()).unwrap();

        insta::assert_json_snapshot!(vec![sk_b58.to_string(), pk_b58.to_string()], @r###"
        [
          "edskRc1okCG3fjFkaDuENVdbepWSsxM3BJCt6FiJZd8xK5tpZEQdHhyvD38T2Z2NKp9NYPF6ixJhrWmYMr1PEc1kVeN4boMhTY",
          "edpktfpdouHjAze9TeFcihdpeMng7FSCWbY4BozpSffZ9z85nyyBBB"
        ]
        "###);

        // testing round trip
        let sk: &[u8] = sk_b58.as_ref();
        let sk: ed25519_compact::SecretKey = ed25519_compact::SecretKey::from_slice(sk).unwrap();
        let sk_b582 = tezos_crypto_rs::hash::SecretKeyEd25519::try_from_bytes(sk.as_ref()).unwrap();
        assert!(sk_b58.to_string().eq(&sk_b582.to_string()));

        let json_str = serde_json_wasm::to_string(&inner).unwrap();
        let json = json_str.clone().into_bytes();
        let hash = Blake2b::from(&json).to_string();

        insta::assert_display_snapshot!(json_str, @r###"{"nonce":777,"content":{"PlacePixel":{"x":1,"y":2,"color":[1,2,3]}}}"###);
        insta::assert_debug_snapshot!(hash, @r###""e379475d1e1109142b0f2ae255d58e98492aff646826fc52d705a80e6b171833""###);

        let nonce = Nonce(777);
        let x: u32 = 1;
        let y: u32 = 2;
        let color_1: u8 = 1;
        let color_2: u8 = 2;
        let color_3: u8 = 3;
        insta::assert_debug_snapshot!(format!("{}{}{}{}{}{}", nonce.to_string(), x, y, color_1, color_2, color_3), @r###""0000030912123""###);

        let message = UserMessage::new(sk, inner);
        insta::assert_json_snapshot!(message, @r###"
        {
          "pkey": {
            "Ed25519": "edpktfpdouHjAze9TeFcihdpeMng7FSCWbY4BozpSffZ9z85nyyBBB"
          },
          "signature": {
            "Ed25519": "edsigtpxbt1mWVGykfTE2D87DybgTY7PmvB4Nhg7N3Xuof6DsvGNwNVsXkWa65SLMsvQfav9FwxcEfnZPCvQiWgUnNFjxvCFwDs"
          },
          "inner": {
            "nonce": 777,
            "content": {
              "PlacePixel": {
                "x": 1,
                "y": 2,
                "color": [
                  1,
                  2,
                  3
                ]
              }
            }
          }
        }
        "###);

        let message_str = r#"{"pkey":{"Ed25519":"edpktfpdouHjAze9TeFcihdpeMng7FSCWbY4BozpSffZ9z85nyyBBB"},"signature":{"Ed25519":"edsigtrE8dQEskw8KQsbZuCGaFBtcTr2NiYeEWKvvuRnJE53fzA3njuCUnyX6JWJbCKz8aT8HgHJjAYfw8ryLPKAQ2Mjn4rc4LL"},"inner":{"nonce":777,"content":{"PlacePixel":{"x":227,"y":357,"color": [0, 1, 2]}}}} "#;
        let _message: UserMessage = serde_json_wasm::from_str(&message_str).unwrap();
    }
}
