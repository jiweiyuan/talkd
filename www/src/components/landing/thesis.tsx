"use client";

import { Reveal } from "./reveal";

export function Thesis() {
  return (
    <Reveal className="py-16">
      <h2 className="text-[2.2rem] font-normal leading-[1.3] mb-8 tracking-tight max-sm:text-[1.6rem]">
        Agents need to talk to each other.
        <br />
        Without you in the middle.
      </h2>
      <p className="text-ink-light mb-5 max-w-[600px]">
        Every multi-agent system hits the same wall:{" "}
        <strong className="text-ink font-semibold">
          how do the agents communicate?
        </strong>{" "}
        HTTP webhooks need servers. Message queues need infrastructure. Shared
        databases need configuration.
      </p>
      <p className="text-ink-light mb-5 max-w-[600px]">
        <code className="font-mono text-[0.82em] bg-green-bg px-1.5 py-0.5 rounded-sm">
          talkd
        </code>{" "}
        is the{" "}
        <strong className="text-ink font-semibold">
          unix philosophy answer
        </strong>
        : a single binary that gives any agent — Claude, GPT, local LLMs,
        scripts — a way to send and receive messages, peer-to-peer, encrypted,
        across any network.
      </p>
      <p className="text-ink-light max-w-[600px]">
        Built on{" "}
        <a
          href="https://iroh.computer"
          className="text-ink underline decoration-ink/25 underline-offset-[3px] hover:decoration-green transition-colors"
        >
          iroh
        </a>{" "}
        (QUIC + NAT traversal + DHT discovery). Agents find each other through
        cryptographic identity, not IP addresses.{" "}
        <strong className="text-ink font-semibold">
          If two agents know each other&apos;s ID, they can talk.
        </strong>
      </p>
    </Reveal>
  );
}
