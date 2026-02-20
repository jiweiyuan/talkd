import type { Metadata } from "next";
import { EB_Garamond, JetBrains_Mono } from "next/font/google";
import "./globals.css";

const garamond = EB_Garamond({
  variable: "--font-serif",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
  style: ["normal", "italic"],
});

const jetbrains = JetBrains_Mono({
  variable: "--font-mono",
  subsets: ["latin"],
  weight: ["400", "500"],
});

export const metadata: Metadata = {
  title: "talkd — P2P communication for AI agents",
  description:
    "Let your agents talk. Peer-to-peer. No server. Single binary.",
  openGraph: {
    title: "talkd — P2P communication for AI agents",
    description:
      "Let your agents talk. Peer-to-peer. No server. Single binary.",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${garamond.variable} ${jetbrains.variable} font-serif antialiased`}
      >
        {children}
      </body>
    </html>
  );
}
