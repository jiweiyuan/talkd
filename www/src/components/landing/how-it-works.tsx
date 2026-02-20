"use client";

import { Reveal, RevealStagger, RevealItem } from "./reveal";

const steps = [
  {
    n: "01",
    label: "Identity",
    text: "ed25519 keypair generated on first run, stored at ~/.talkd/identity",
  },
  {
    n: "02",
    label: "Discovery",
    text: "NodeId published to Pkarr DHT (built on BitTorrent mainline)",
  },
  {
    n: "03",
    label: "Connect",
    text: "iroh's QUIC transport handles NAT traversal via relay servers",
  },
  {
    n: "04",
    label: "Gossip",
    text: "iroh-gossip pub/sub broadcasts messages to all channel peers",
  },
  {
    n: "05",
    label: "Persist",
    text: "JSON files on disk with per-subscriber cursors. Simple, inspectable, reliable",
  },
];

export function HowItWorks() {
  return (
    <Reveal className="py-16">
      <h2 className="font-mono text-base uppercase tracking-[0.15em] text-ink-muted text-center mb-10">
        Under the hood
      </h2>
      <RevealStagger className="max-w-[520px] mx-auto">
        {steps.map((s) => (
          <RevealItem
            key={s.n}
            className="flex gap-5 py-4 border-b border-border items-baseline last:border-0 transition-all hover:pl-2"
          >
            <span className="font-mono text-[1rem] text-green font-medium shrink-0">
              {s.n}
            </span>
            <span className="text-ink-light text-lg">
              <strong className="text-ink font-semibold">{s.label}</strong> —{" "}
              {s.text}
            </span>
          </RevealItem>
        ))}
      </RevealStagger>
    </Reveal>
  );
}
