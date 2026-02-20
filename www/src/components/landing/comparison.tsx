"use client";

import { Reveal } from "./reveal";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

const rows = [
  { label: "Server needed", talkd: "No", redis: "Yes", http: "Yes" },
  { label: "NAT traversal", talkd: "Built-in", redis: "N/A", http: "Port forwarding" },
  { label: "Encryption", talkd: "E2E (QUIC)", redis: "Optional TLS", http: "TLS" },
  { label: "Setup time", talkd: "0 min", redis: "10+ min", http: "Varies" },
  { label: "Dependencies", talkd: "None", redis: "Runtime", http: "Runtime" },
  { label: "Cross-network", talkd: "Yes (DHT)", redis: "VPN needed", http: "Public IP" },
];

export function Comparison() {
  return (
    <Reveal className="py-16">
      <h2 className="font-mono text-base uppercase tracking-[0.15em] text-ink-muted text-center mb-8">
        Why not
      </h2>
      <Table className="font-sans text-[1rem]">
        <TableHeader>
          <TableRow className="border-b-2">
            <TableHead className="text-ink font-semibold text-[1rem]" />
            <TableHead className="text-ink font-semibold text-[1rem]">
              talkd
            </TableHead>
            <TableHead className="text-ink font-semibold text-[1rem]">
              Redis/RabbitMQ
            </TableHead>
            <TableHead className="text-ink font-semibold text-[1rem]">
              HTTP webhooks
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.map((r) => (
            <TableRow
              key={r.label}
              className="transition-colors hover:bg-green-bg"
            >
              <TableCell className="text-ink-muted">{r.label}</TableCell>
              <TableCell className="text-green font-medium">
                {r.talkd}
              </TableCell>
              <TableCell className="text-ink-light">{r.redis}</TableCell>
              <TableCell className="text-ink-light">{r.http}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </Reveal>
  );
}
