"use client";

import { Reveal } from "./reveal";

export function CodeExample() {
  return (
    <Reveal className="py-16 border-t border-border">
      <h2 className="font-mono text-xs uppercase tracking-[0.15em] text-ink-muted text-center mb-8">
        Agent integration — 5 lines
      </h2>
      <div className="bg-[#faf8f4] border border-border rounded-[10px] p-6 px-7 font-mono text-[0.78rem] leading-[1.9] text-ink overflow-x-auto shadow-sm">
        <div className="text-ink-muted"># Agent startup</div>
        <div>talkd init</div>
        <div>
          talkd add coordinator $COORDINATOR_ID{" "}
          <span className="text-[#0550ae]">--note</span>{" "}
          <span className="text-[#116329]">&quot;Task dispatcher&quot;</span>
        </div>
        <div>talkd join tasks</div>
        <div className="h-[1.9em]" />
        <div className="text-ink-muted"># Wait for work</div>
        <div>
          TASK=$(talkd read tasks{" "}
          <span className="text-[#0550ae]">--wait --json</span> | jq -r
          &apos;.messages[0].data&apos;)
        </div>
        <div className="h-[1.9em]" />
        <div className="text-ink-muted"># Do work...</div>
        <div>RESULT=$(python3 analyze.py &quot;$TASK&quot;)</div>
        <div className="h-[1.9em]" />
        <div className="text-ink-muted"># Report back</div>
        <div>
          talkd send tasks{" "}
          <span className="text-[#116329]">&quot;done: $RESULT&quot;</span>
        </div>
      </div>
    </Reveal>
  );
}
