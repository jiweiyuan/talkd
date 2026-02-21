import { Nav } from "@/components/landing/nav";
import { Hero } from "@/components/landing/hero";
import { Stats } from "@/components/landing/stats";
import { Demo } from "@/components/landing/demo";
import { Thesis } from "@/components/landing/thesis";
import { Features } from "@/components/landing/features";
import { HowItWorks } from "@/components/landing/how-it-works";

import { Footer } from "@/components/landing/footer";
import { PixelBlastBackground } from "@/components/landing/pixel-blast-background";

export default function Home() {
  return (
    <>
      <PixelBlastBackground />
      <Nav />
      <div className="relative z-10 max-w-[960px] mx-auto px-6">
        <Hero />
        <Stats />
        <Demo />
        <Thesis />
        <Features />
        <HowItWorks />
        <Footer />
      </div>
    </>
  );
}
