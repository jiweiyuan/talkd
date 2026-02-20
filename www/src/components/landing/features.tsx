"use client";

import { motion, useInView } from "framer-motion";
import { useRef } from "react";

/* ── Feature data ─────────────────────────────────────────────────── */

const features = [
  {
    tag: "channels",
    desc: "Join a channel, broadcast to everyone. Coordinator talks to workers, workers talk back.",
  },
  {
    tag: "DM",
    desc: "Private 1:1 communication. ECDH key agreement — only the two agents can read it.",
  },
  {
    tag: "--json",
    desc: "Every command supports --json output. Agents parse structured data, not terminal strings.",
  },
  {
    tag: "--file",
    desc: "Send files up to 3MB inline. Auto-saved on receive.",
  },
  {
    tag: "contacts",
    desc: "Name your peers with notes. Agents search by skill.",
  },
  {
    tag: "identity",
    desc: "Each agent gets an ed25519 keypair. Your ID is your public key. No registration, no server.",
  },
];

/*
 * Diagonal stagger order for a 2-col × 3-row grid:
 *
 *   [0]  [1]        diag 0, diag 1
 *   [2]  [3]   →    diag 1, diag 2
 *   [4]  [5]        diag 2, diag 3
 *
 * diag index = row + col → cards on the same diagonal enter together.
 */
function diagDelay(index: number): number {
  const col = index % 2;
  const row = Math.floor(index / 2);
  const diag = row + col; // 0‥3
  return diag * 0.12; // 120 ms between diagonals
}

/* ── Section header ───────────────────────────────────────────────── */

function SectionHeader() {
  const ref = useRef<HTMLDivElement>(null);
  const inView = useInView(ref, { once: true, margin: "-40px" });

  return (
    <div ref={ref} className="text-center mb-10">
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={inView ? { opacity: 1, y: 0 } : {}}
        transition={{ duration: 0.6 }}
        className="inline-flex items-center gap-3"
      >
        <motion.span
          className="block w-8 h-[1px] bg-green"
          initial={{ scaleX: 0 }}
          animate={inView ? { scaleX: 1 } : {}}
          transition={{ duration: 0.5, delay: 0.2 }}
          style={{ transformOrigin: "right" }}
        />
        <h2 className="font-mono text-base uppercase tracking-[0.15em] text-ink-muted">
          Built for agents
        </h2>
        <motion.span
          className="block w-8 h-[1px] bg-green"
          initial={{ scaleX: 0 }}
          animate={inView ? { scaleX: 1 } : {}}
          transition={{ duration: 0.5, delay: 0.2 }}
          style={{ transformOrigin: "left" }}
        />
      </motion.div>
    </div>
  );
}

/* ── Ambient glow (slow breathing loop behind the grid) ───────────── */

function AmbientGlow() {
  return (
    <motion.div
      className="pointer-events-none absolute inset-0 z-0"
      initial={{ opacity: 0 }}
      animate={{
        opacity: [0.35, 0.7, 0.35],
      }}
      transition={{
        duration: 6,
        repeat: Infinity,
        ease: "easeInOut" as const,
      }}
      style={{
        background:
          "radial-gradient(ellipse 80% 50% at 50% 45%, rgba(22,163,74,0.06) 0%, transparent 70%)",
      }}
    />
  );
}

/* ── Single feature card ──────────────────────────────────────────── */

function FeatureCard({
  tag,
  desc,
  index,
  parentInView,
}: {
  tag: string;
  desc: string;
  index: number;
  parentInView: boolean;
}) {
  const delay = diagDelay(index);

  return (
    <motion.div
      initial={{ opacity: 0, y: 20, scale: 0.97 }}
      animate={
        parentInView
          ? { opacity: 1, y: 0, scale: 1 }
          : {}
      }
      transition={{
        duration: 0.5,
        delay,
        ease: [0.21, 0.47, 0.32, 0.98],
      }}
      className="p-5 rounded-lg border border-border bg-paper"
    >
      <h3 className="font-mono text-[1rem] text-green uppercase tracking-wider mb-2">
        {tag}
      </h3>
      <p className="text-[1.05rem] text-ink-light leading-relaxed">{desc}</p>
    </motion.div>
  );
}

/* ── Main export ──────────────────────────────────────────────────── */

export function Features() {
  const gridRef = useRef<HTMLDivElement>(null);
  const gridInView = useInView(gridRef, { once: true, margin: "-60px" });

  return (
    <section className="py-16 relative">
      <SectionHeader />

      {/* Ambient breathing glow */}
      <AmbientGlow />

      {/* Card grid */}
      <div
        ref={gridRef}
        className="relative z-10 grid grid-cols-2 gap-4 max-sm:grid-cols-1"
      >
        {features.map((f, i) => (
          <FeatureCard
            key={f.tag}
            tag={f.tag}
            desc={f.desc}
            index={i}
            parentInView={gridInView}
          />
        ))}
      </div>
    </section>
  );
}
