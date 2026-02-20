import { Github, BookOpen, Tag } from "lucide-react";

const links = [
  {
    label: "GitHub",
    href: "https://github.com/jiweiyuan/talkd",
    icon: Github,
  },
  {
    label: "Documentation",
    href: "/docs",
    icon: BookOpen,
  },
  {
    label: "Releases",
    href: "https://github.com/jiweiyuan/talkd/releases",
    icon: Tag,
  },
];

export function Footer() {
  return (
    <footer className="py-16 pt-16 border-t border-border text-center">
      <div className="flex justify-center gap-8 mb-5">
        {links.map((l) => (
          <a
            key={l.label}
            href={l.href}
            className="inline-flex items-center gap-1.5 text-ink-muted no-underline font-serif text-[1.1rem] hover:text-green transition-colors"
          >
            <l.icon size={16} strokeWidth={1.5} />
            {l.label}
          </a>
        ))}
      </div>
      <p className="text-ink-muted text-[1rem] font-sans opacity-50">
        MIT License. Built with Rust + iroh.
      </p>
    </footer>
  );
}
