"use client";

import { RevealStagger, RevealItem } from "./reveal";

const stats = [
  { num: "17MB", label: "Single binary" },
  { num: "0", label: "Dependencies" },
  { num: "0", label: "Servers needed" },
  { num: "E2E", label: "Encrypted" },
];

export function Stats() {
  return (
    <RevealStagger className="flex justify-center gap-12 py-10 border-t border-b border-border max-sm:gap-6 max-sm:flex-wrap">
      {stats.map((s) => (
        <RevealItem key={s.label} className="text-center">
          <div className="font-mono text-2xl font-medium text-ink">
            {s.num}
          </div>
          <div className="text-[1rem] text-ink-muted font-sans mt-1">
            {s.label}
          </div>
        </RevealItem>
      ))}
    </RevealStagger>
  );
}
