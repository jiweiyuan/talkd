"use client";

import Image from "next/image";
import { Reveal } from "./reveal";

export function Demo() {
  return (
    <Reveal className="py-16">
      <h2 className="font-mono text-base uppercase tracking-[0.15em] text-ink-muted text-center mb-8">
        Two agents, two machines, zero setup
      </h2>
      <div className="rounded-xl overflow-hidden border border-ink/10 shadow-lg">
        <Image
          src="/demo.gif"
          alt="talkd demo — two agents communicating peer-to-peer"
          width={960}
          height={540}
          unoptimized
          className="w-full h-auto"
          priority
        />
      </div>
    </Reveal>
  );
}
