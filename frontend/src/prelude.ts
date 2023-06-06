import { Buffer } from "buffer";
globalThis.Buffer = Buffer;
import "./style.css";
import * as tweetnacl from "tweetnacl";
import * as blake from "blakejs";
import * as bs58check from "bs58check";
import { InMemorySigner } from "@taquito/signer";
import { TezosToolkit } from "@taquito/taquito";
import {secret_path, twitter_path} from "./paths";

export const PREFIX = {
  tz1: new Uint8Array([6]),
  edsk: new Uint8Array([13, 15, 58, 7]),
};

/**
 * Hash the string representation of the payload, returns the b58 reprensentation starting with the given prefix
 * @param prefix the prefix of your hash
 * @returns
 */
export const toB58Hash = (prefix: Uint8Array, payload: string) => {
  const blakeHash = blake.blake2b(payload, undefined, 32);
  const tmp = new Uint8Array(prefix.length + blakeHash.length);
  tmp.set(prefix);
  tmp.set(blakeHash, prefix.length);
  const b58 = bs58check.encode(Buffer.from(tmp));
  return b58;
};

export const newPrivateKey = () => {
  // Generate a random 32-byte seed
  var seed = new Uint8Array(32);
  crypto.getRandomValues(seed);

  // Generate the key pair from the seed
  const keyPair = tweetnacl.sign.keyPair.fromSeed(seed);
  const secretKey = Buffer.from(keyPair.secretKey).toString("base64");
  const publicKey = Buffer.from(keyPair.publicKey).toString("base64");
  return toB58Hash(PREFIX.edsk, secretKey);
};

async function main() {
  let secret = window.localStorage.getItem(secret_path);
  let twitterHandle = window.localStorage.getItem(twitter_path); 
  if (!secret) {
    secret = newPrivateKey();
    window.localStorage.setItem(secret_path, secret);
  }
  
  if(twitterHandle && secret) {
    window.location.href = "/desktop.html";
  }
   
  const signer = new InMemorySigner(secret);

  const tezos = new TezosToolkit("https://mainnet.api.tez.ie/"); 

  tezos.setProvider({
    signer,
  });

  const enterElement = document.getElementById('enter');
  enterElement.addEventListener('click', function(event) {
    let attested = window.localStorage.getItem(twitter_path);
    if(!attested) {
        alert("Register on twitter first, then play");
        event.preventDefault(); 
    }
  });

  const linkElement = document.getElementById('attest');

  // Add event listener for click event
  linkElement.addEventListener('click', async function(event) {
    event.preventDefault(); 
    const pkh = await tezos.wallet.pkh();
    let twitterHandle = (document.getElementById("twitter-handle") as any).value;
    if(!twitterHandle) {
        alert("Bruh, I said enter your twitter handle in the text box");
        return 
    } else {
       const signature = await signer.sign(`1:${twitterHandle}:${pkh}`);
       console.log(signature);
       const ending = `Entering%20the%20%23PixelWar%20at%20https%3A%2F%2Fpixel-war.rollups.dev%20as%20${pkh}.%0A%0ASignature%3A%20${signature.sig}`; 
       const url= `https://twitter.com/intent/tweet?text=${ending}`;
       window.open(url, '_blank');
       window.localStorage.setItem(twitter_path, twitterHandle);
    }
  });
}

main();
