"use client";

import { useEffect, useRef } from "react";
import { usePathname } from "next/navigation";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

export function WindowResizer() {
  const pathname = usePathname();
  const lastSize = useRef({ width: 0, height: 0 });
  const isUpdating = useRef(false);

  useEffect(() => {
    const isTauri = typeof window !== "undefined" && Boolean((window as unknown as { __TAURI_INTERNALS__?: boolean }).__TAURI_INTERNALS__);
    if (!isTauri) {
      return;
    }

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

        if (width > 100 && height > 100) {
          if (Math.abs(width - lastSize.current.width) > 5 || Math.abs(height - lastSize.current.height) > 5) {
            isUpdating.current = true;
            await appWindow.setSize(new LogicalSize(Math.ceil(width), Math.ceil(height)));
            lastSize.current = { width, height };
            isUpdating.current = false;
          }
        }
      } catch (error) {
        console.error("Failed to resize window:", error);
        isUpdating.current = false;
      }
    };

    const observer = new ResizeObserver(() => {
      requestAnimationFrame(() => updateWindowSize());
    });

    observer.observe(document.body);
    if (document.body.firstElementChild) {
      observer.observe(document.body.firstElementChild);
    }
    observer.observe(document.documentElement);

    updateWindowSize();

    const timers = [
      setTimeout(updateWindowSize, 50),
      setTimeout(updateWindowSize, 150),
      setTimeout(updateWindowSize, 300),
      setTimeout(updateWindowSize, 500)
    ];

    return () => {
      observer.disconnect();
      timers.forEach(clearTimeout);
    };
  }, [pathname]);

  return null;
}
