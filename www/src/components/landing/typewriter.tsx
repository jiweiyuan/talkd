"use client";

import { useState, useEffect, useRef } from "react";
import { useInView } from "framer-motion";

interface Line {
  type: "cmd" | "out" | "blank";
  text: string;
}

interface TerminalTypewriterProps {
  lines: Line[];
  startDelay?: number;
  id: string;
  title: string;
}

export function TerminalTypewriter({
  lines,
  startDelay = 0,
  id,
  title,
}: TerminalTypewriterProps) {
  const ref = useRef<HTMLDivElement>(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });
  const [visibleLines, setVisibleLines] = useState<
    { text: string; done: boolean }[]
  >(lines.map(() => ({ text: "", done: false })));
  const started = useRef(false);

  useEffect(() => {
    if (!isInView || started.current) return;
    started.current = true;

    let delay = startDelay;
    const timeouts: NodeJS.Timeout[] = [];

    lines.forEach((line, i) => {
      if (line.type === "blank") {
        timeouts.push(
          setTimeout(() => {
            setVisibleLines((prev) => {
              const next = [...prev];
              next[i] = { text: " ", done: true };
              return next;
            });
          }, delay)
        );
        delay += 100;
        return;
      }

      if (line.type === "cmd") {
        // Typewriter effect
        const chars = line.text.split("");
        chars.forEach((_, ci) => {
          timeouts.push(
            setTimeout(() => {
              setVisibleLines((prev) => {
                const next = [...prev];
                next[i] = {
                  text: line.text.slice(0, ci + 1),
                  done: ci === chars.length - 1,
                };
                return next;
              });
            }, delay + ci * 35)
          );
        });
        delay += chars.length * 35 + 200;
      } else {
        // Output: instant appear
        timeouts.push(
          setTimeout(() => {
            setVisibleLines((prev) => {
              const next = [...prev];
              next[i] = { text: line.text, done: true };
              return next;
            });
          }, delay)
        );
        delay += 150;
      }
    });

    return () => timeouts.forEach(clearTimeout);
  }, [isInView, lines, startDelay]);

  return (
    <div ref={ref} className="rounded-[10px] overflow-hidden bg-[#faf8f4] border border-border shadow-sm" id={id}>
      <div className="flex items-center gap-2 px-4 py-2.5 border-b border-border">
        <div className="w-2.5 h-2.5 rounded-full bg-[#ff5f57]" />
        <div className="w-2.5 h-2.5 rounded-full bg-[#febc2e]" />
        <div className="w-2.5 h-2.5 rounded-full bg-[#28c840]" />
        <span className="ml-2 font-mono text-sm text-ink-muted">
          {title}
        </span>
      </div>
      <div className="p-4 px-5 font-mono text-[0.8rem] leading-[1.9] text-ink min-h-[220px]">
        {lines.map((line, i) => {
          const v = visibleLines[i];
          if (!v?.text && !v?.done) return <div key={i} className="h-[2em]" />;

          if (line.type === "blank") {
            return <div key={i} className="h-[1.6em]" />;
          }

          if (line.type === "cmd") {
            return (
              <div key={i}>
                <span className="text-green">$ </span>
                <span>{v.text}</span>
                {!v.done && (
                  <span
                    className="text-green"
                    style={{ animation: "cursor-blink 1s step-end infinite" }}
                  >
                    █
                  </span>
                )}
              </div>
            );
          }

          return (
            <div key={i} className="text-ink-muted">
              {v.text}
            </div>
          );
        })}
      </div>
    </div>
  );
}
