"use client";

import { useEffect, useRef } from "react";
import { usePathname } from "next/navigation";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

export function WindowResizer() {
  const pathname = usePathname();
  const lastSize = useRef({ width: 0, height: 0 });
  const isUpdating = useRef(false);
  const minSize = useRef<{ width: number; height: number }>({ width: 896, height: 600 });

  useEffect(() => {
    const isTauri = typeof window !== "undefined" && Boolean((window as unknown as { __TAURI_INTERNALS__?: boolean }).__TAURI_INTERNALS__);
    if (!isTauri) {
      return;
    }

    // Route-based design minimums
    const getDesignMinSize = (path: string) => {
      // Default main window target per design: 896x600
      const defaults = { width: 896, height: 600 };
      // Tray or compact views (if any): 300x400
      if (path.startsWith("/tray")) return { width: 300, height: 400 };
      // Future compact views could be listed here
      return defaults;
    };
    minSize.current = getDesignMinSize(pathname || "/");

    const updateWindowSize = async () => {
      if (isUpdating.current) return;
      
      try {
        const appWindow = getCurrentWindow();
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

        const targetWidth = Math.max(width, minSize.current.width);
        const targetHeight = Math.max(height, minSize.current.height);

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
