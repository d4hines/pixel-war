use tezos_smart_rollup::{prelude::*, storage::path::*};

use lib::public_key_hash::PublicKeyHash;
use lib::receipt::Receipt;
use lib::message::PlacePixel;
use lib::{account::Account, error::*, nonce::Nonce};

const ACCOUNTS: RefPath = RefPath::assert_from(b"/accounts");
const RECEIPTS: RefPath = RefPath::assert_from(b"/receipts");

/// Compute the paths for the different fields of a tweet
///
/// The field_path should start with slash
fn pixel_path(x: u32, y: u32) -> Result<OwnedPath> {
    let path: Vec<u8> = format!("/image/{}/{}", x, y).into();
    OwnedPath::try_from(path).map_err(Error::from)
}

/// Compute the paths for the different fields of an account
///
/// The field_path should start with slash
fn account_field_path(public_key_hash: &PublicKeyHash, field_path: &str) -> Result<OwnedPath> {
    let public_key_hash: Vec<u8> = format!("/{}", public_key_hash.to_string()).into();
    let public_key_hash = OwnedPath::try_from(public_key_hash).map_err(Error::from)?;
    let public_key_hash = concat(&ACCOUNTS, &public_key_hash).map_err(Error::from)?;

    let field_path: Vec<u8> = field_path.into();
    let field_path = OwnedPath::try_from(field_path).map_err(Error::from)?;
    concat(&public_key_hash, &field_path).map_err(Error::from)
}

/// Compute the path /accounts/{tz1...}/nonce
fn nonce_path(public_key_hash: &PublicKeyHash) -> Result<OwnedPath> {
    account_field_path(public_key_hash, "/nonce")
}


/// Compute the path of the different field of a receipt
fn receipt_field_path(receipt: &Receipt, field_path: &str) -> Result<OwnedPath> {
    let receipt_path = format!("/{}", receipt.hash().to_string());
    let receipt_path = OwnedPath::try_from(receipt_path).map_err(Error::from)?;
    let receipt_path = concat(&RECEIPTS, &receipt_path)?;

    let field_path: Vec<u8> = field_path.into();
    let field_path = OwnedPath::try_from(field_path).map_err(Error::from)?;
    concat(&receipt_path, &field_path).map_err(Error::from)
}

/// Compute the path of the success field of a receipt
fn receipt_success_path(receipt: &Receipt) -> Result<OwnedPath> {
    receipt_field_path(receipt, "/success")
}

///  Check if a path exists
pub fn exists<R: Runtime>(host: &mut R, path: &impl Path) -> Result<bool> {
    let exists = Runtime::store_has(host, path)?
        .map(|_| true)
        .unwrap_or_default();
    Ok(exists)
}

/// Read an u64 from a given path
/// If the data does not exist, it returns the default value of an u64
pub fn read_u64<R: Runtime>(host: &mut R, path: &impl Path) -> Result<Option<u64>> {
    let is_exists = exists(host, path)?;
    if !is_exists {
        return Ok(None);
    }

    let mut buffer = [0_u8; 8];
    match host.store_read_slice(path, 0, &mut buffer) {
        Ok(8) => Ok(Some(u64::from_be_bytes(buffer))),
        _ => Err(Error::StateDeserializarion),
    }
}

/// Store an u64 at a given path
fn store_u64<'a, R: Runtime>(host: &mut R, path: &impl Path, u64: &'a u64) -> Result<&'a u64> {
    let data = u64.to_be_bytes();
    let data = data.as_slice();

    host.store_write(path, data, 0)
        .map_err(Error::from)
        .map(|_| u64)
}

/// Stores a boolean at a given path
fn store_bool<R: Runtime>(host: &mut R, path: &impl Path, bool: bool) -> Result<()> {
    let data = match bool {
        true => [0x01],
        false => [0x00],
    };

    host.store_write(path, &data, 0)
        .map_err(Error::from)
        .map(|_| ())
}

/// Read the account of the user
pub fn read_account<R: Runtime>(host: &mut R, public_key_hash: PublicKeyHash) -> Result<Account> {
    let nonce_path = nonce_path(&public_key_hash)?;
    let nonce = read_u64(host, &nonce_path)?.unwrap_or_default();
    Ok(Account {
        public_key_hash,
        nonce: Nonce(nonce),
    })
}

/// Store an account to the location /account/{tz...}
///
/// Only the nonce is stored
pub fn store_account<'a, R: Runtime>(host: &mut R, account: &'a Account) -> Result<&'a Account> {
    let Account {
        nonce,
        public_key_hash,
    } = account;
    let nonce_path = nonce_path(public_key_hash)?;
    let _ = store_u64(host, &nonce_path, &nonce.0)?;
    Ok(account)
}

/// Store a tweet to the location /tweets/{tz...}
pub fn store_pixel<'a, R: Runtime>(
    host: &mut R,
    place_pixel: &'a PlacePixel,
) -> Result<&'a PlacePixel> {
    let PlacePixel {
        x,
        y,
        color,
    } = place_pixel;
    debug_msg!(host, "Placing pixel: {:?},{:?}:{:?}\n", x, y, color) ;
    let path : OwnedPath = pixel_path(x.clone(), y.clone())?;
    host.store_write(&path, color, 0)
    .map_err(Error::from)
    .map(|_| place_pixel)?;
    Ok(place_pixel)
}


// Stores a receipt under /receipt/{hash}
pub fn store_receipt<'a, R: Runtime>(host: &mut R, receipt: &'a Receipt) -> Result<&'a Receipt> {
    let success_path = receipt_success_path(receipt)?;

    store_bool(host, &success_path, receipt.success())?;

    Ok(receipt)
}
