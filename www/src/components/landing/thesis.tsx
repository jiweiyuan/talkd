"use client";

import { Reveal } from "./reveal";

export function Thesis() {
  return (
    <Reveal className="py-16">
      <h2 className="text-[2.2rem] font-normal leading-[1.3] mb-8 tracking-tight max-sm:text-[1.6rem]">
        The agentic web needs a telephone system,
        <br />
        not another intercom.
        <sup className="text-[0.5em] text-ink-light align-super">
          <a
            href="https://projectnanda.org/assets/NANDA.pdf"
            className="text-ink-light hover:text-green transition-colors no-underline"
            target="_blank"
            rel="noopener noreferrer"
          >
            1
          </a>
        </sup>
      </h2>
      <p className="text-ink-light mb-5 max-w-[600px]">
        You set up{" "}
        <a
          href="https://github.com/openclaw/openclaw"
          className="text-ink underline decoration-ink/25 underline-offset-[3px] hover:decoration-green transition-colors"
        >
          OpenClaw
        </a>
        . It&apos;s extraordinary — an AI that runs on your machine, answers on
        your terms, acts on your behalf. Then you want it to talk to your
        partner&apos;s agent. Or a coding agent on another server. Or a booking
        agent you&apos;ve never heard of.
      </p>
      <p className="text-ink-light mb-5 max-w-[600px]">
        Email, Slack, Discord — every obvious fix puts a platform and
        complicated integration in the middle. Your agent deserves a direct
        line.
      </p>
      <p className="text-ink-light mb-5 max-w-[600px]">
        <code className="font-mono text-[0.82em] bg-green-bg px-1.5 py-0.5 rounded-sm">
          talkd
        </code>{" "}
        is the telephone system model: any agent can reach any other agent with
        nothing but an ID.
      </p>
      <ul className="text-ink-light max-w-[600px] space-y-2 mb-5 list-none pl-0">
        <li>
          <strong className="text-ink font-semibold">
            Identity without registration.
          </strong>{" "}
          Cryptographic keypair. No accounts, no platform.
        </li>
        <li>
          <strong className="text-ink font-semibold">
            Discovery without a directory.
          </strong>{" "}
          BitTorrent mainline DHT. No central registry.
        </li>
        <li>
          <strong className="text-ink font-semibold">
            Communication without infrastructure.
          </strong>{" "}
          QUIC transport, NAT traversal, relay fallback. No server to deploy.
        </li>
      </ul>
      <p className="text-ink-light mb-5 max-w-[600px]">
        <strong className="text-ink font-semibold">
          One binary. Zero dependencies. If two agents know each other&apos;s
          ID, they can talk. That&apos;s it.
        </strong>
      </p>
      <p className="text-ink-faint text-[0.78em] max-w-[600px]">
        <sup>1</sup> The &ldquo;telephone system vs. intercom&rdquo; framing is
        inspired by{" "}
        <a
          href="https://projectnanda.org/assets/NANDA.pdf"
          className="text-ink-light underline decoration-ink/15 underline-offset-[3px] hover:decoration-green transition-colors"
          target="_blank"
          rel="noopener noreferrer"
        >
          NANDA
        </a>{" "}
        (Networked AI Agents in Decentralized Architecture), an MIT initiative
        led by Prof. Ramesh Raskar addressing the four choke points of the open
        agentic web.
      </p>
    </Reveal>
  );
}
