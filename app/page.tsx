"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { API } from "@/lib/api";

export default function Home() {
  const router = useRouter();

  useEffect(() => {
    const route = async () => {
      try {
        const hasConfig = await API.hasConfig();

        if (!hasConfig) {
          const hasPermission = await API.checkPermissions();
          if (!hasPermission) {
            router.replace("/onboarding");
            return;
          }
          router.replace("/onboarding/llm");
          return;
        }

        const hasPermission = await API.checkPermissions();
        if (!hasPermission) {
          router.replace("/onboarding");
          return;
        }

        router.replace("/rules");
      } catch (error) {
        console.error("Failed to route app flow", error);
        router.replace("/onboarding");
      }
    };

    route();
  }, [router]);

  return (
    <div className="h-screen w-full flex items-center justify-center text-sm text-muted-foreground">
      正在初始化...
    </div>
  );
}
