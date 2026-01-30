"use client";

import { Button } from "@/components/ui/button";
import { ModeToggle } from "@/components/mode-toggle";
import { useRouter } from "next/navigation";
import { ChevronRight } from "lucide-react";

export default function Home() {
  const router = useRouter();

  return (
    <main className="relative min-h-screen flex flex-col items-center justify-center px-4 py-8 bg-background text-foreground">
      {/* Top-right theme toggle */}
      <div className="absolute top-4 right-4 z-10">
        <ModeToggle />
      </div>

      {/* Main card */}
      <div className="w-full max-w-md bg-card rounded-2xl shadow-lg p-8 space-y-8">
        <section className="flex flex-col gap-4">
          <h3 className="text-sm font-medium text-muted-foreground text-center">Development Testing</h3>
          <Button 
            variant="outline" 
            onClick={() => router.push("/onboarding")}
            className="w-full gap-2 h-12"
          >
            Test Onboarding Flow <ChevronRight className="w-4 h-4" />
          </Button>
        </section>
      </div>
    </main>
  );
}
