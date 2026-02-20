"use client";

import { Reveal } from "./reveal";
import { TerminalTypewriter } from "./typewriter";

const termALines = [
  { type: "cmd" as const, text: "talkd init" },
  { type: "out" as const, text: "Identity ready" },
  { type: "blank" as const, text: "" },
  { type: "cmd" as const, text: "talkd create ops" },
  { type: "out" as const, text: 'Created channel "ops"' },
  { type: "out" as const, text: "Invite ticket:" },
  { type: "out" as const, text: "mjavkqadiaqeb6aw4jlg..." },
  { type: "blank" as const, text: "" },
  { type: "cmd" as const, text: 'talkd send ops "task done"' },
  { type: "out" as const, text: "Sent (delivered to 1 recipient)" },
  { type: "blank" as const, text: "" },
  { type: "cmd" as const, text: "talkd read ops" },
  { type: "out" as const, text: "[14:30:05] b3f1bc20: got it, next task?" },
];

const termBLines = [
  { type: "cmd" as const, text: "talkd init" },
  { type: "out" as const, text: "Identity ready" },
  { type: "blank" as const, text: "" },
  { type: "cmd" as const, text: "talkd join mjavkqadiaqeb6aw4jlg..." },
  { type: "out" as const, text: 'Joined channel "ops"' },
  { type: "blank" as const, text: "" },
  { type: "cmd" as const, text: "talkd read ops --wait" },
  { type: "out" as const, text: "[14:30:00] a3f1bc20: task done" },
  { type: "blank" as const, text: "" },
  { type: "cmd" as const, text: 'talkd send ops "got it, next task?"' },
  { type: "out" as const, text: "Sent (delivered to 1 recipient)" },
];

export function Demo() {
  return (
    <Reveal className="py-16">
      <h2 className="font-mono text-base uppercase tracking-[0.15em] text-ink-muted text-center mb-8">
        Two agents, two machines, zero setup
      </h2>
      <div className="grid grid-cols-2 gap-4 max-sm:grid-cols-1">
        <TerminalTypewriter
          id="term-a"
          title="Agent A — San Francisco"
          lines={termALines}
          startDelay={300}
        />
        <TerminalTypewriter
          id="term-b"
          title="Agent B — Tokyo"
          lines={termBLines}
          startDelay={1800}
        />
      </div>
    </Reveal>
  );
}
