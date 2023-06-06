use lib::dac::{reveal_loop, V0SliceContentPage, MAX_PAGE_SIZE};
use lib::message::Message;
use lib::message::{Content, UserMessage};
use lib::public_key::PublicKey;
use lib::public_key_hash::PublicKeyHash;

use lib::constants::SEQUENCER_PK;
// src/lib.rs
use storage::{read_account, store_account};
use tezos_smart_rollup::core_unsafe::PREIMAGE_HASH_SIZE;
use tezos_smart_rollup::michelson::ticket;
use tezos_smart_rollup::storage::path::{OwnedPath};
use tezos_smart_rollup::{kernel_entry, prelude::*};

mod stages;
mod storage;
mod upgrade;

use lib::error::*;
use stages::{place_pixel, read_input, verify_nonce, verify_signature};


/// A step is processing only one message from the inbox
///
/// It will execute several sub steps:
/// - verify the signature of the message
/// - verify the nonce of the message
/// - handle the message
fn step<R: Runtime>(host: &mut R, message: UserMessage, _level: u32) -> Result<()> {
    let public_key = message.public_key();
    let public_key_hash = PublicKeyHash::from(public_key);
    host.write_debug("Message is deserialized\n");

    let inner = verify_signature(message)?;
    host.write_debug("Signature is correct\n");

    // Verify the nonce
    let account = read_account(host, public_key_hash)?;
    let content = verify_nonce(inner, account.nonce())?;
    let account = account.increment_nonce();
    let _ = store_account(host, &account)?;

    // Interpret the message
    match content {
        Content::PlacePixel(post_tweet) => place_pixel(host, &account, post_tweet)?,
    };

    Ok(())
}

fn handle_txs<Host: Runtime>(
    level: u32,
) -> impl FnMut(&mut Host, V0SliceContentPage) -> std::result::Result<(), &'static str> {
    move |host, page| {
        let content: &[u8] = page.as_ref();
        let content = std::str::from_utf8(content).unwrap();
        let message: UserMessage = serde_json_wasm::from_str(content).unwrap();
        // FIXME:
        step(host, message, level).unwrap();
        Ok(())
    }
}

/// Process all the inbox
///
/// Read a message, process the error of the read message
/// If the message is correctly deserialized it continue the execution
/// Then all the errors, will be stored in a receipt
/// Continue until the inbox is emptied
///
/// This function stop its execution when a RuntimeError happens
///
/// TODO: it can count ticks and reboot the kernel between two inbox message
fn execute<R: Runtime>(host: &mut R) -> Result<()> {
    let message = read_input::<ticket::BytesTicket>(host);
    match message {
        Err(ReadInputError::EndOfInbox) => Ok(()),
        Err(ReadInputError::TimeToReboot) => Ok(()),
        Err(ReadInputError::Runtime(err)) => Err(Error::Runtime(err)),
        Err(err) => {
            debug_msg!(host, "Error while reading input: {:?}\n", err);
            execute(host)
        }
        Ok((None, level))  => {
            execute(host)
        },
        Ok((Some(message), level)) => {
            let Message {
                signature,
                unprefixed_merkle_root,
            } = message;

            let pk = PublicKey::from_b58(&SEQUENCER_PK).unwrap();
            debug_msg!(host, "verifying sequencer signature: {:?}\n", signature);
            signature.verify(&pk, &unprefixed_merkle_root)?;
            debug_msg!(host, "sequencer signature is valid\n");

            // this is done because I don't know how to use Serde properly
            let mut root_hash = [0; PREIMAGE_HASH_SIZE];
            root_hash[1..].copy_from_slice(&unprefixed_merkle_root);

            // Support 3 levels of hashes pages, and then bottom layer of content.
            const MAX_DAC_LEVELS: usize = 4;

            let mut buffer = [0; MAX_PAGE_SIZE * MAX_DAC_LEVELS];
            let mut handle_txs = handle_txs(level);

            match reveal_loop(
                host,
                0,
                &root_hash,
                buffer.as_mut_slice(),
                MAX_DAC_LEVELS,
                &mut handle_txs,
            ) {
                Err(err) => Err(Error::GenericError(String::from(err))),
                Ok(()) => Ok(()),
            }
        }
    }
}

pub fn entry<R: Runtime>(host: &mut R) {
    debug_msg!(host, "Executing kernel: {}\n", env!("GIT_HASH"));
    let greeting_path: OwnedPath = "/greeting".as_bytes().to_vec().try_into().unwrap();
    let _ = Runtime::store_write(host, &greeting_path, "hello world".as_bytes(), 0);
    match execute(host) {
        Ok(_) => {}
        Err(err) => debug_msg!(host, "{}\n", &err.to_string()),
    }
}

kernel_entry!(entry);



#[test]
fn test() {
   const USER_MESSAGES: &[&str] = &[
    r#"{
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
        }"#,
    // r#"{
    //       "pkey": {
    //         "Ed25519": "edpktfpdouHjAze9TeFcihdpeMng7FSCWbY4BozpSffZ9z85nyyBBB"
    //       },
    //       "signature": {
    //         "Ed25519": "edsigtZtyBeW9ZVcchuV3ModHtfGgH8zpdMCdm9gTnpBJoXWr1w44uuysVMX8WJyf3dEKpgKz53EpeirUDFK7seQ4iRHF25FSSw"
    //       },
    //       "inner": {
    //         "nonce": 777,
    //         "content": {
    //           "PlacePixel": {
    //             "x": 1,
    //             "y": 2,
    //             "rgba": [
    //               1,
    //               2,
    //               3,
    //               4
    //             ]
    //           }
    //         }
    //       }
    //     }"#,
]; 
    // let preimage_dir = Path::new("/home/d4hines/repos/tezos-place/rollup/wasm_2_0_0");
    // if !preimage_dir.is_dir() {
    //     fs::create_dir_all(preimage_dir).unwrap();
    // }

    // let save_preimages = |hash: PreimageHash, preimage: Vec<u8>| {
    //     let name = hex::encode(hash.as_ref());
    //     let path = preimage_dir.join(name);

    //     if let Err(e) = fs::write(&path, preimage) {
    //         eprintln!("Failed to write preimage to {:?} due to {}.", path, e);
    //     }
    // };

    let content: Vec<Vec<u8>> = USER_MESSAGES
        .iter()
        .map(|x| {
            let bytes: &[u8] = x.as_ref();
            bytes.to_vec()
        })
        .collect();
    let mut host = tezos_smart_rollup_mock::MockHost::default();
    let root_hash: PreimageHash = prepare_preimages(content, |_hash, page| {
        host.set_preimage(page);
    })
    .unwrap();
    let mut unprefixed_merkel_root: [u8; 32] = [0; 32];
    unprefixed_merkel_root.copy_from_slice(&root_hash.as_ref()[1..]);

    let sk = tezos_crypto_rs::hash::SecretKeyEd25519::from_base58_check(
            "edskRrdh2fnaZv2sDYB9Lv6dmNPSeMAMBtRK9E4Ap85ea8pQfaDvxisnhHsCGihvpLBDnbBdwjBPL1nWtJuzWhfXR3LErGut7d",
        )
    .unwrap();

     let sk = sk.as_ref().as_slice();
    let sk = ed25519_compact::SecretKey::from_slice(sk).unwrap();

    let message = Message::new(sk, unprefixed_merkel_root);

    MockHost::add_external(&mut host, message);

    let _level = MockHost::run_level(&mut host, entry);
    let greeting = "hello world".as_bytes();
    let greeting_path: OwnedPath = "/greeting".as_bytes().to_vec().try_into().unwrap();
    let greeting_read = MockHost::store_read(&mut host, &greeting_path, 0, greeting.len()).unwrap();
    assert!(greeting == greeting_read);

    let first_pixel_path: RefPath = RefPath::assert_from(b"/image/1/2");
    let first_pixel: Vec<u8> = MockHost::store_read(&mut host, &first_pixel_path, 0, 4).unwrap();
    assert!(first_pixel == vec![1, 2, 3]);
    ()
}
