"use client";

import dynamic from "next/dynamic";

const PixelBlast = dynamic(() => import("@/components/PixelBlast"), {
  ssr: false,
});

export function PixelBlastBackground() {
  return (
    <div className="fixed inset-0 z-0 opacity-40 pointer-events-auto">
      <PixelBlast
        variant="circle"
        pixelSize={4}
        color="#16a34a"
        className=""
        style={{}}
        patternScale={2}
        patternDensity={0.8}
        speed={0.3}
        edgeFade={0.4}
        enableRipples={true}
        rippleSpeed={0.3}
        rippleThickness={0.12}
        rippleIntensityScale={0.8}
        transparent={true}
      />
    </div>
  );
}
