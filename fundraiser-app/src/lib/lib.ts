export type ContractStatus =
  | "funding"
  | "locked"
  | "resolved_successful"
  | "resolved_unsuccessful";

export type ContractState = {
  status: ContractStatus;
  oracleTimeout: number;
  resolutionDate: Date;
  refundLockPeriod: number;
  amountRaised: number;
};

export const CONTRACT_ADDRESS = "KT1KS7Nk5CfCaL3PGaDGzkToTCg3wCAvzWJW";

export const getContractData = async (): Promise<ContractState> => {
  const [contractData, contractStorage] = await Promise.all(
    [
      fetch(`https://api.tzkt.io/v1/contracts/${CONTRACT_ADDRESS}`, {
        cache: "no-store",
      }),
      fetch(`https://api.tzkt.io/v1/contracts/${CONTRACT_ADDRESS}/storage`, {
        cache: "no-store",
      }),
    ].map((p) => p.then((x) => x.json()))
  );
  const status = Object.keys(contractStorage.status)[0] as ContractStatus;
  const resolutionDate =
    status === "locked" ? new Date(contractStorage.status.locked) : new Date();
  console.log(resolutionDate);
  return {
    status,
    oracleTimeout: Number(contractStorage.oracle_timeout),
    resolutionDate,
    refundLockPeriod: Number(contractStorage.refund_lock_period),
    amountRaised: contractData.balance,
  };
};
