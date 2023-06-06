use crate::{storage::store_pixel, upgrade};

use lib::constants::{L1_GOVERNANCE_CONTRACT_ADDRESS, MAGIC_BYTE};

use lib::{
    account::Account,
    message::{Content, Inner, Message, PlacePixel},
    nonce::Nonce,
};
use tezos_crypto_rs::hash::ContractKt1Hash;

use tezos_smart_rollup::{
    core_unsafe::PREIMAGE_HASH_SIZE,
    inbox::{InboxMessage, InternalInboxMessage},
    michelson::{ticket, Michelson, MichelsonBytes},
    prelude::*,
};

use lib::error::*;
use lib::message::UserMessage;

fn vec_to_array_ref<'a, const N: usize>(vec: &'a Vec<u8>) -> Option<&'a [u8; N]> {
    if vec.len() == N {
        Some((&vec[..]).try_into().ok()?)
    } else {
        None
    }
}

pub fn read_input<Expr: Michelson>(
    host: &mut impl Runtime,
) -> std::result::Result<(Option<Message>, u32), ReadInputError> {
    let input = host.read_input().map_err(ReadInputError::Runtime)?;
    match input {
        None => Err(ReadInputError::EndOfInbox),
        Some(message) => {
            debug_msg!(host, "read input: {:?}\n", message);
            match InboxMessage::<ticket::BytesTicket>::parse(message.as_ref()) {
                Ok(parsed_msg) => match parsed_msg {
                    (remaining, InboxMessage::Internal(msg)) => {
                        assert!(remaining.is_empty());
                        match msg {
                            InternalInboxMessage::StartOfLevel => {
                                host.write_debug(format!("Start of Level\n").as_str());
                                Ok((None, message.level))
                            }
                            InternalInboxMessage::InfoPerLevel(info) => {
                                host.write_debug(format!("Info: {:?}\n", info).as_str());
                                Ok((None, message.level))
                            }
                            InternalInboxMessage::EndOfLevel => {
                                host.write_debug(format!("End of Level\n").as_str());
                                Ok((None, message.level))
                            }
                            InternalInboxMessage::Transfer(transfer) => {
                                host.write_debug(format!("Transfer: {:?}\n", transfer).as_str());
                                let upgrade_contract: ContractKt1Hash =
                                    tezos_crypto_rs::hash::ContractKt1Hash::from_base58_check(
                                        L1_GOVERNANCE_CONTRACT_ADDRESS,
                                    )
                                    .unwrap();
                                if transfer.sender == upgrade_contract {
                                    let ticket: ticket::BytesTicket = transfer.payload;
                                    let MichelsonBytes(data) = ticket.contents();
                                    let data: &Vec<u8> = data;
                                    let root_hash: &[u8; PREIMAGE_HASH_SIZE] =
                                        vec_to_array_ref(data).unwrap();
                                    let root_hash_hex = hex::encode(root_hash);
                                    host.write_debug(
                                        format!("Received upgrade hash: {}\n", root_hash_hex)
                                            .as_str(),
                                    );
                                    upgrade::install_kernel(host, root_hash).unwrap();
                                    host.write_debug(format!("Upgrade complete\n").as_str());
                                    host.mark_for_reboot().unwrap();
                                    Ok((None, message.level))  
                                } else {
                                    Ok((None, message.level)) 
                                }
                            }
                        }
                    }
                    (remaining, InboxMessage::External(data)) => {
                        assert!(remaining.is_empty());
                        match data {
                            [MAGIC_BYTE, ..] => {
                                let bytes = data.iter().skip(1).copied().collect();
                                let str = String::from_utf8(bytes)
                                    .map_err(ReadInputError::FromUtf8Error)?;
                                let msg: Message = serde_json_wasm::from_str(&str)
                                    .map_err(ReadInputError::SerdeJson)?;
                                Ok((Some(msg), message.level))
                            }
                            _ => {
                                debug_msg!(host, "External message not intended for this rollup\n");
                                Err(ReadInputError::NotATzwitterMessage)
                            }
                        }
                    }
                },
                Err(err) => Err(ReadInputError::GenericError(format!(
                    "Unknown error: {:?}",
                    err
                ))),
            }
        }
    }
}

/// Verify the signature of a message
///
/// Returns the inner message
pub fn verify_signature(message: UserMessage) -> Result<Inner> {
    let signature = message.signature();
    let pkey = message.public_key();
    let inner = message.inner();
    let hash = inner.hash();

    signature.verify(pkey, hash.as_ref())?;
    let UserMessage { inner, .. } = message;
    Ok(inner)
}

/// Verify the nonce of the inner message
///
/// If the nonce is correct the content of the inner is returned
pub fn verify_nonce(inner: Inner, _nonce: &Nonce) -> Result<Content> {
    Ok(inner.content)
    // FIXME: implement this later
    // let next_nonce = nonce.next();
    // let inner_nonce = inner.nonce();
    // if &next_nonce == inner_nonce {
    //     let Inner { content, .. } = inner;
    //     Ok(content)
    // } else {
    //     Err(Error::InvalidNonce)
    // }
}

/// Create a new tweet from the PostTweet request
/// Save the tweet to the durable state
/// And add a tweet entry to the user account
pub fn place_pixel<R: Runtime>(
    host: &mut R,
    _account: &Account,
    place_pixel: PlacePixel,
) -> Result<()> {
    store_pixel(host, &place_pixel)?;
    Ok(())
}
