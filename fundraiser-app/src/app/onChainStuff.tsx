"use client";

import { useState, useEffect } from "react";
import { ContractAbstraction, TezosToolkit, Wallet } from "@taquito/taquito";
import { BeaconWallet } from "@taquito/beacon-wallet";
import { NetworkType } from "@airgap/beacon-dapp";
import { BigNumber } from "bignumber.js";
import { CONTRACT_ADDRESS, ContractState, ContractStatus } from "../lib/lib";
import * as Sentry from "@sentry/nextjs";

const Tezos = new TezosToolkit("https://mainnet.tezos.marigold.dev/");

const wallet = new BeaconWallet({
  name: "Pixel War Fundraiser",
  preferredNetwork: NetworkType.MAINNET,
});

Tezos.setWalletProvider(wallet);

const showAmount = (amount: number) => `${amount / 1000000}ꜩ`;

const makeErrorMessage = (msg: string) =>
  `There was an error submitting the operation. Please try again or contact danhines09 on Twitter. Error: ${msg}`;

const dateDiff = (date1: Date, date2: Date) => {
  const rawTimeDiff = (date2 as any) - (date1 as any);
  const timeDiff = Math.abs(rawTimeDiff);
  const sign = rawTimeDiff < 0 ? "-" : "";
  const hours = Math.floor(timeDiff / (1000 * 60 * 60));
  const minutes = Math.floor((timeDiff % (1000 * 60 * 60)) / (1000 * 60));
  const seconds = Math.floor((timeDiff % (1000 * 60)) / 1000);
  const displayNumber = (number: number) =>
    number < 10 ? `0${number}` : number;
  return [sign, ...[hours, minutes, seconds].map(displayNumber)];
};

type PledgeState =
  | { pledgeStatus: "pledged"; amount: number }
  | { pledgeStatus: "refund-requested"; amount: number; refundUnlockDate: Date }
  | { pledgeStatus: "not-pledged" };

type WalletContract = ContractAbstraction<Wallet>;

type ConnectedState = {
  status: "connected";
  userAddress: string;
  pledgeState: PledgeState;
  contract: WalletContract;
};

type DisconnectedState = {
  status: "disconnected";
};

type State = ConnectedState | DisconnectedState;

export const CountDown = ({ until }: { until: Date }) => {
  const [now, setNow] = useState(new Date());
  useEffect(() => {
    const myInterval = setInterval(() => {
      setNow(new Date());
    }, 1000);
    return () => clearInterval(myInterval);
  }, []);
  const [sign, hours, minutes, seconds] = dateDiff(now, until);
  return (
    <b>
      {sign}
      {hours}:{minutes}:{seconds}
    </b>
  );
};

export function OnChainStuff({
  contractState,
}: {
  contractState: ContractState;
}) {
  const [message, setMessage] = useState("");
  const [txs, setTxs] = useState<number[]>([]);
  const pushTxs = (x: number) => setTxs([...txs, x]);
  const [state, setState] = useState<State>({
    status: "disconnected",
  });
  const [now, setNow] = useState(new Date());

  const connect = async () => {
    const transaction = Sentry.startTransaction({
      name: "Connect",
    });
    Sentry.configureScope((scope) => {
      scope.setSpan(transaction);
    });
    try {
      setMessage("Loading...");
      if (!(await wallet.client.getActiveAccount())) {
        await wallet.requestPermissions({
          network: { type: NetworkType.MAINNET },
        });
      }
      const userAddress = await wallet.getPKH();
      const contract = await Tezos.wallet.at(CONTRACT_ADDRESS);
      const storage = (await contract.storage()) as any;
      const userLedgerEntry = await storage.ledger
        .get(userAddress)
        .catch(() => null);
      const status = "connected";
      const pledgeState: PledgeState = (() => {
        if (userLedgerEntry) {
          const amount = (userLedgerEntry["0"] as BigNumber).toNumber();
          if (userLedgerEntry["1"] !== null && "Some" in userLedgerEntry["1"]) {
            const refundUnlockDate = new Date(userLedgerEntry["1"]["Some"]);
            return {
              pledgeStatus: "refund-requested",
              amount,
              refundUnlockDate,
            };
          } else {
            return {
              pledgeStatus: "pledged",
              amount,
            };
          }
        } else {
          return { pledgeStatus: "not-pledged" };
        }
      })();
      setState({
        status,
        contract,
        userAddress,
        pledgeState,
      });
      setMessage("");
    } finally {
      transaction.finish();
    }
  };

  const submitOperation = async (method: any, amount?: number) => {
    try {
      setMessage("Awaiting wallet interaction...");
      const op = await (amount ? method().send({ amount }) : method().send());
      setMessage("Awaiting blockchain confirmation...");
      await op.confirmation();
      setMessage("");
    } catch (err: any) {
      setMessage(makeErrorMessage(err.toString()));
    }
  };

  const GivePledge = ({ state }: { state: ConnectedState }) => (
    <form
      id="pledge-form"
      onSubmit={async (e) => {
        e.preventDefault();
        const transaction = Sentry.startTransaction({
          name: "Give Pledge",
        });

        Sentry.configureScope((scope) => {
          scope.setSpan(transaction);
        });

        try {
          const formData = new FormData(e.target as HTMLFormElement);
          const formJson = Object.fromEntries(formData.entries());
          const amount = Number(formJson.amount);
          await submitOperation(state.contract.methods.give_pledge, amount);
          const prevPledgeAmount =
            state.pledgeState.pledgeStatus === "pledged"
              ? state.pledgeState.amount
              : 0;
          const mutezAmount = amount * 1000000;
          setState({
            status: "connected",
            userAddress: state.userAddress,
            contract: state.contract,
            pledgeState: {
              pledgeStatus: "pledged",
              amount: Number(mutezAmount + prevPledgeAmount),
            },
          });
          pushTxs(mutezAmount);
        } finally {
          transaction.finish();
        }
      }}
    >
      <label>
        <span>Enter a pledge amount in ꜩ:</span>
        <input type="number" step="0.000001" name="amount" defaultValue={0} />
      </label>
      <input type="submit" value="Give Pledge" />
    </form>
  );

  const getRefund = async (state: ConnectedState) => {
    await submitOperation(state.contract.methods.get_refund);
  };

  const Connected = ({ state }: { state: ConnectedState }) => {
    switch (contractState.status) {
      case "funding":
        switch (state.pledgeState.pledgeStatus) {
          case "not-pledged":
            return <GivePledge state={state} />;
          case "pledged":
            const pledgedAmount = Number(state.pledgeState.amount);
            return (
              <>
                <p>Your pledge: {showAmount(pledgedAmount)}</p>
                <GivePledge state={state} />
                <p>
                  Or, initiate a refund (available to finalize after 2 hours):
                  &nbsp;
                  <button
                    onClick={async () => {
                      const transaction = Sentry.startTransaction({
                        name: "Initiate Refund",
                      });

                      Sentry.configureScope((scope) => {
                        scope.setSpan(transaction);
                      });
                      try {
                        await getRefund(state);
                        const refundUnlockDate = new Date(
                          new Date().getTime() +
                            contractState.refundLockPeriod * 1000
                        );
                        setTimeout(() => {
                          setNow(new Date());
                        }, refundUnlockDate.getTime() - new Date().getTime());

                        setState({
                          status: "connected",
                          userAddress: state.userAddress,
                          contract: state.contract,
                          pledgeState: {
                            pledgeStatus: "refund-requested",
                            amount: pledgedAmount,
                            refundUnlockDate,
                          },
                        });
                      } finally {
                        transaction.finish();
                      }
                    }}
                  >
                    Get Refund
                  </button>
                </p>
              </>
            );
          case "refund-requested":
            if (now > state.pledgeState.refundUnlockDate) {
              const amountRefunded = state.pledgeState.amount;
              return (
                <>
                  <p>Your refund is ready for withdraw.</p>
                  <button
                    onClick={async () => {
                      const transaction = Sentry.startTransaction({
                        name: "Finalize Refund",
                      });

                      Sentry.configureScope((scope) => {
                        scope.setSpan(transaction);
                      });
                      try {
                        await getRefund(state);
                        pushTxs(amountRefunded * -1);
                        setState({
                          status: "connected",
                          userAddress: state.userAddress,
                          contract: state.contract,
                          pledgeState: {
                            pledgeStatus: "not-pledged",
                          },
                        });
                      } finally {
                        transaction.finish();
                      }
                    }}
                  >
                    Finalize Refund
                  </button>
                </>
              );
            } else {
              return (
                <p>
                  Your refund of {showAmount(state.pledgeState.amount)} will be
                  ready for withdraw in{" "}
                  <CountDown until={state.pledgeState.refundUnlockDate} />.
                </p>
              );
            }
        }
      case "locked":
        // Shouldn't be reachable
        return <></>;
      case "resolved_successful":
        // Shouldn't be reachable
        return <></>;
      case "resolved_unsuccessful":
        const showRefund =
          state.pledgeState.pledgeStatus === "pledged" ||
          state.pledgeState.pledgeStatus === "refund-requested";
        return (
          <>
            <p>Fundraiser resolved unsuccessfully.</p>
            {showRefund && (
              <button
                onClick={async () => {
                  await getRefund(state);
                  setState({
                    ...state,
                    pledgeState: { pledgeStatus: "not-pledged" },
                  });
                }}
              >
                Get Instant Refund
              </button>
            )}
          </>
        );
    }
  };
  const amountRaised = txs.reduce(
    (prev, curr) => prev + curr,
    contractState.amountRaised
  );
  return (
    <div className="on-chain-stuff">
      <h4>Amount raised so far: {showAmount(amountRaised)}</h4>
      {(() => {
        switch (true) {
          case contractState.status === "resolved_successful":
            return (
              <p>Fundraiser resolved successfully! Enjoy your Pixel War!</p>
            );
          case contractState.status === "locked":
            const oracleTimeout = new Date(
              contractState.resolutionDate.getTime() +
                contractState.oracleTimeout * 1000
            );
            const oracleText =
              new Date() > contractState.resolutionDate ? (
                <>
                  <p>Waiting on Oracle for resolution.</p>
                  <p>
                    Time until automatic resolution:{" "}
                    <CountDown until={oracleTimeout} />.
                  </p>
                </>
              ) : (
                <>
                  <p>Funds locked while Daniel is working.</p>
                  <p>
                    Oracle can resolve the commitment in:{" "}
                    <CountDown until={contractState.resolutionDate} />
                  </p>
                </>
              );
            return <>{oracleText}</>;
          case message === "Loading...":
            return <></>;
          case state.status === "disconnected":
            return <button onClick={connect}>Connect with Beacon</button>;
          case state.status === "connected":
            return <Connected state={state as ConnectedState} />;
        }
      })()}
      {message ? <span>{message}</span> : <></>}
    </div>
  );
}
