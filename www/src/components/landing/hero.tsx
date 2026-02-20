"use client";

import { motion } from "framer-motion";
import { useState } from "react";

const fadeUp = {
  initial: { opacity: 0, y: 16 },
  animate: { opacity: 1, y: 0 },
};

export function Hero() {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(
      "curl -sSL https://raw.githubusercontent.com/jiweiyuan/talkd/main/install.sh | bash"
    );
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section className="pt-24 pb-16 text-center">
      <motion.div
        {...fadeUp}
        transition={{ duration: 0.7, delay: 0 }}
        className="font-serif text-[1.5rem] uppercase tracking-[0.15em] text-green mb-5"
      >
        P2P for AI Agents
      </motion.div>

      <motion.h1
        {...fadeUp}
        transition={{ duration: 0.7, delay: 0.12 }}
        className="text-[4.5rem] font-normal tracking-tight leading-none mb-6 max-md:text-[3rem]"
      >
        Let your agents <em>talk</em>
      </motion.h1>

      <motion.p
        {...fadeUp}
        transition={{ duration: 0.7, delay: 0.24 }}
        className="text-[1.35rem] text-ink-light max-w-[480px] mx-auto mb-12 leading-snug"
      >
        Peer-to-peer communication for AI agents. No server. No config. Just a
        single binary and they find each other.
      </motion.p>

      <motion.button
        {...fadeUp}
        transition={{ duration: 0.7, delay: 0.36 }}
        onClick={handleCopy}
        className="inline-flex items-center gap-3 bg-ink text-white rounded-lg px-6 py-3.5 font-mono text-[1rem] cursor-pointer border-none hover:-translate-y-0.5 transition-transform"
      >
        <span className="text-white/70 select-none">$</span>
        <span>curl -sSL ... | bash</span>
        <span className="text-white/60 text-[1rem] font-mono ml-2">
          {copied ? "copied!" : "click to copy"}
        </span>
      </motion.button>

      <motion.div
        {...fadeUp}
        transition={{ duration: 0.7, delay: 0.48 }}
        className="mt-5 flex justify-center gap-6 text-[1rem]"
      >
        <a
          href="https://github.com/jiweiyuan/talkd/releases/latest"
          className="text-green no-underline font-serif hover:underline"
        >
          Download v0.3.1
        </a>
        <a
          href="https://github.com/jiweiyuan/talkd/releases"
          className="text-ink-muted no-underline font-serif hover:text-ink transition-colors"
        >
          All releases
        </a>
      </motion.div>
    </section>
  );
}
