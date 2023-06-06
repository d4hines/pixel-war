type storage = key

type parameter = {
  target : address;
  payload_hash : bytes;
  dictator_signature : signature;
}

(* Main access point that dispatches to the entrypoints according to
   the smart contract parameter.
   
   example invokation for tests:
   (for data "asdf")
  
   target : KT1M9ir3wBLCKuHVewuyFAJWAmsWC3S7hdSe
   payload_hash : 0xb91349ff7c99c3ae3379dd49c2f3208e202c95c0aac5f97bb24ded899e9a1e83 (remove 0x in better-call.dev)
   dictator_signature : edsigu6NKVkvxYg6oBokhJrVKvD5imtyke7qtaX658MiYYxzY2HqogJFfuoWUFyP8CwhVobKZxZ5eXncXKWu53JGP5XiY7C7Daf *)
let main (action : parameter) (storage : storage) : operation list * storage =
  let signature_valid = Crypto.check storage action.dictator_signature action.payload_hash in
  let target_contract : (bytes ticket) contract = Tezos.get_contract action.target in
  let txs : operation list = if signature_valid then
        let ticket = Tezos.create_ticket action.payload_hash 1n |> Option.unopt in
        let tx = Tezos.transaction ticket 0tz target_contract in
        [tx]
  else []
  in txs, storage
