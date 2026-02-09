"use client";

import { useEffect, useRef } from "react";
import { usePathname } from "next/navigation";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { getWindowSizing } from "@/lib/window-size";

export function WindowResizer() {
  const pathname = usePathname();
  const lastSize = useRef({ width: 0, height: 0 });
  const isUpdating = useRef(false);
  const minSize = useRef<{ width: number; height: number }>({ width: 896, height: 600 });
  const fixedSize = useRef<{ width: number; height: number } | null>(null);

  useEffect(() => {
    const isTauri = typeof window !== "undefined" && Boolean((window as unknown as { __TAURI_INTERNALS__?: boolean }).__TAURI_INTERNALS__);
    if (!isTauri) {
      return;
    }

    const sizing = getWindowSizing(pathname || "/");
    minSize.current = sizing.min;
    fixedSize.current = sizing.fixed;

    const updateWindowSize = async () => {
      if (isUpdating.current) return;
      
      try {
        const appWindow = getCurrentWindow();
        let targetWidth = minSize.current.width;
        let targetHeight = minSize.current.height;

        if (fixedSize.current) {
          targetWidth = fixedSize.current.width;
          targetHeight = fixedSize.current.height;
        } else {
          const body = document.body;
          const doc = document.documentElement;
          if (!body || !doc) return;

          const width = Math.max(
            body.scrollWidth,
            doc.scrollWidth,
            body.offsetWidth,
            doc.offsetWidth,
            body.clientWidth,
            doc.clientWidth
          );
          const height = Math.max(
            body.scrollHeight,
            doc.scrollHeight,
            body.offsetHeight,
            doc.offsetHeight,
            body.clientHeight,
            doc.clientHeight
          );

          targetWidth = Math.max(width, minSize.current.width);
          targetHeight = Math.max(height, minSize.current.height);
        }

        if (targetWidth > 100 && targetHeight > 100) {
          if (Math.abs(targetWidth - lastSize.current.width) > 5 || Math.abs(targetHeight - lastSize.current.height) > 5) {
            isUpdating.current = true;
            await appWindow.setMinSize(new LogicalSize(minSize.current.width, minSize.current.height));
            await appWindow.setSize(new LogicalSize(Math.ceil(targetWidth), Math.ceil(targetHeight)));
            lastSize.current = { width: targetWidth, height: targetHeight };
            isUpdating.current = false;
          }
        }
      } catch (error) {
        console.error("Failed to resize window:", error);
        isUpdating.current = false;
      }
    };

    const raf = requestAnimationFrame(() => {
      updateWindowSize();
      // One additional pass to catch late layout (fonts, images, async data).
      setTimeout(updateWindowSize, 120);
    });

    return () => {
      cancelAnimationFrame(raf);
    };
  }, [pathname]);

  return null;
}
