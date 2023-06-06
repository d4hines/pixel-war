import { CONTRACT_ADDRESS, getContractData } from "@/lib/lib";
import { OnChainStuff } from "./onChainStuff";

export default async function Home() {
  const contractData = await getContractData();
  return (
    <article>
      <header>
        <h1>
          Tezos Pixel War Season 1 was <em>awesome.</em>
        </h1>
      </header>
      <p>
        On May 18th, 2023 I launched an{" "}
        <a target="_blank" href="https://en.wikipedia.org/wiki/R/place">
          r/place
        </a>
        -inspired real-time collaborative pixel canvas. Over the course of 48
        hours,
        <b> 184 players</b> made a combined <b>697,132 transactions</b>!
      </p>

      <p>
        I compiled the entire history into an{" "}
        <a
          target="_blank"
          href="https://objkt.com/asset/KT1TJQrX2Uu5x8pGs6DfJq9qtstKmAPhYxpy/5"
        >
          interactive NFT on Objkt
        </a>
        .
      </p>

      <h2>Check it out👇</h2>
      <iframe src="/nft.html" frameBorder="0"></iframe>

      <h2>Now I'm raising money to build Season 2.</h2>
      <p>
        Building this thing on bleeding-edge tech isn't easy. Help me justify
        the time invested in this community project.
      </p>
      <p>
        To raise the money, I've built a Kickstarter-like smart contract. Check
        out the{" "}
        <a
          target="_blank"
          href="https://github.com/d4hines/fundraising-contract"
        >
          Ligo source code
        </a>{" "}
        and the{" "}
        <a
          target="_blank"
          href={`https://better-call.dev/mainnet/${CONTRACT_ADDRESS}/code`}
        >
          deployed contract on BCD
        </a>
        .
      </p>
      <p> Here's how it works:</p>
      <ul>
        <li>
          During the fundraising period, anyone can pledge any amount of tez
          toward the project.
        </li>
        <li>
          At any point during the fundraising period, you can cancel your
          pledge. After 2 hours, your pledge is fully refunded.
        </li>
        <li>
          If enough money is raised, I'll post a commitment on-chain to ship
          Season 2 within 5 weeks{" "}
          <small>(I can only do part-time work on it)</small>.
        </li>
        <li>
          If I can get enough money pledged, I commit to shipping Season 2 by
          July 1st, 2023.
        </li>
        <li>
          At the end of 5 weeks a neutral third-party oracle (the inimitable{" "}
          <a target="_blank" href="https://twitter.com/claudebarde">
            Claude Barde
          </a>
          ) will decide if I kept my commitment.
          <ul>
            <li>If I did, I keep the money. Enjoy your Pixel War.</li>
            <li>If I did not, you can collect a full refund. No sweat.</li>
          </ul>
        </li>
      </ul>
      <h3>If I can raise 15000ꜩ, I'll run a Season 2 with:</h3>
      <ul>
        <li>
          <b>A perpetual canvas.</b> Play as long as you want. It's a permanent
          fixture in the Tezos art community.
        </li>
        <li>
          <b>Capture any moment in an NFT.</b> Focus on the time, area, and
          players you want, with royalty splits.
        </li>
        <li>
          <b>Proper wallet integration</b>. Use your normal Tezos wallet, but
          with the same fast, feeless transaction experience.
        </li>
        <li>
          <b>Improved UI</b>. Proper color picker. See who's playing with you.
        </li>
        <li>
          <b>Anti-bot measures</b> in the form of a small fee to play (1ꜩ =
          10000 pixels).
        </li>
      </ul>

      <h3>You in? </h3>
      <p>Connect your wallet and contribute a pledge of any amount.</p>
      <small>
        (Disclaimer: This contract has not been audited! Do your own research
        and use with caution!)
      </small>
      <OnChainStuff contractState={contractData} />
    </article>
  );
}
