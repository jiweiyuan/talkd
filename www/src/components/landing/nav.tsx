"use client";

import { Github, BookOpen, Tag } from "lucide-react";

export function Nav() {
  return (
    <nav className="sticky top-0 z-50 bg-paper/90 backdrop-blur-sm border-b border-border">
      <div className="max-w-[960px] mx-auto px-6 py-3 flex justify-between items-center">
        <a href="/" className="font-mono font-medium text-lg no-underline tracking-tight">
          talkd
        </a>
        <div className="flex gap-5 items-center">
          <a
            href="/docs"
            className="inline-flex items-center gap-1.5 text-ink-muted no-underline text-[1rem] font-serif hover:text-ink transition-colors"
          >
            <BookOpen size={16} strokeWidth={1.5} />
            Docs
          </a>
          <a
            href="https://github.com/jiweiyuan/talkd/releases"
            className="inline-flex items-center gap-1.5 text-ink-muted no-underline text-[1rem] font-serif hover:text-ink transition-colors"
          >
            <Tag size={16} strokeWidth={1.5} />
            Releases
          </a>
          <a
            href="https://github.com/jiweiyuan/talkd"
            className="inline-flex items-center gap-1.5 text-ink-muted no-underline text-[1rem] font-serif hover:text-ink transition-colors"
          >
            <Github size={16} strokeWidth={1.5} />
            GitHub
          </a>
        </div>
      </div>
    </nav>
  );
}
