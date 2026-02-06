"use client";

import React, { useEffect, useMemo, useState } from "react";
import { motion } from "framer-motion";
import { API, AppConfig } from "@/lib/api";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useRouter } from "next/navigation";

type ScanPhase = "scanning" | "analyzing" | "generated" | "error";

export default function ScanOnboardingPage() {
  const router = useRouter();
  const [phase, setPhase] = useState<ScanPhase>("scanning");
  const [progress, setProgress] = useState(0);
  const [errorMessage, setErrorMessage] = useState("");

  const statusText = useMemo(() => {
    if (phase === "error") return "生成失败";
    if (phase === "generated") return "生成完成";
    if (phase === "analyzing") return "分析中...";
    return "扫描中...";
  }, [phase]);

  useEffect(() => {
    let active = true;
    let localProgress = 0;
    const timer = setInterval(() => {
      if (!active) return;
      const increment = Math.random() * 6 + 4;
      localProgress = Math.min(localProgress + increment, 88);
      setProgress(Math.round(localProgress));
      if (localProgress > 45) {
        setPhase("analyzing");
      }
    }, 650);

    const runScan = async () => {
      try {
        const inputSources = await API.getSystemInputSources();
        const generatedRules = await API.scanAndPredict(inputSources);
        if (!active) return;
        setProgress(100);
        setPhase("generated");
        setErrorMessage("");
        const config: AppConfig = {
          version: 1,
          global_switch: true,
          default_input: "keep",
          general: {
            auto_start: true,
            show_menu_bar_status: true,
            hide_dock_icon: false,
          },
          rules: generatedRules,
        };
        await API.saveConfig(config);
      } catch (error) {
        if (!active) return;
        const message =
          typeof error === "string"
            ? error
            : error instanceof Error
              ? error.message
              : "";
        setErrorMessage(message || "扫描失败，请检查 LLM 配置");
        setPhase("error");
      } finally {
        clearInterval(timer);
      }
    };

    runScan();

    return () => {
      active = false;
      clearInterval(timer);
    };
  }, []);

  return (
    <div className="w-full h-full min-h-screen bg-white dark:bg-zinc-900 flex items-center justify-center">
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className={cn(
          "relative flex flex-col items-center",
          "w-[344px] h-[500px]",
          "border border-[#e4e4e7] dark:border-zinc-800",
          "rounded-[24px]",
          "shadow-[0px_25px_50px_-12px_rgba(0,0,0,0.25)]",
          "bg-white dark:bg-zinc-900",
          "px-8 pt-8 pb-[126px]"
        )}
      >
        <div className="relative h-20 w-20 rounded-full bg-[#eff6ff] overflow-hidden">
          <div className="absolute -left-[7px] -top-[7px] h-[95px] w-[95px] bg-[#2b7fff1a] opacity-95" />
          <svg
            width="40"
            height="40"
            viewBox="0 0 40 40"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            className="absolute left-5 top-5"
          >
            <path d="M20 8.33332C20.002 7.6667 19.8706 7.00642 19.6137 6.39132C19.3567 5.77621 18.9794 5.21869 18.5038 4.75156C18.0282 4.28443 17.4641 3.9171 16.8445 3.6712C16.2248 3.4253 15.5623 3.30578 14.8959 3.31967C14.2294 3.33356 13.5724 3.48059 12.9636 3.75209C12.3548 4.0236 11.8064 4.41411 11.3507 4.90066C10.895 5.3872 10.5412 5.95996 10.3101 6.58524C10.079 7.21052 9.97529 7.8757 10.005 8.54165C9.02536 8.79354 8.11586 9.26507 7.34541 9.9205C6.57496 10.5759 5.96376 11.3981 5.5581 12.3247C5.15244 13.2514 4.96296 14.2581 5.00401 15.2688C5.04506 16.2795 5.31556 17.2676 5.79503 18.1583C4.95199 18.8432 4.28907 19.7237 3.86393 20.7232C3.43878 21.7227 3.26429 22.811 3.35563 23.8933C3.44698 24.9756 3.80139 26.0192 4.38804 26.9334C4.97468 27.8475 5.7758 28.6044 6.72169 29.1383C6.60489 30.042 6.67459 30.9601 6.92651 31.8359C7.17842 32.7116 7.60719 33.5264 8.18633 34.2299C8.76548 34.9334 9.48271 35.5108 10.2937 35.9263C11.1047 36.3417 11.9923 36.5865 12.9017 36.6455C13.811 36.7045 14.7228 36.5765 15.5807 36.2693C16.4386 35.9622 17.2244 35.4824 17.8896 34.8596C18.5548 34.2368 19.0853 33.4843 19.4483 32.6484C19.8113 31.8126 19.9991 30.9112 20 30V8.33332Z" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M15 21.6667C16.3993 21.1744 17.6211 20.2783 18.5111 19.0917C19.4011 17.905 19.9192 16.4811 20 15" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M10.005 8.54167C10.0379 9.34789 10.2655 10.1342 10.6683 10.8333" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M5.79498 18.16C6.09988 17.9117 6.42615 17.6908 6.76998 17.5" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M10 30C8.85141 30.0006 7.72214 29.7043 6.72169 29.14" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M20 21.6667H26.6667" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M20 30H30C30.8841 30 31.7319 30.3512 32.357 30.9763C32.9821 31.6014 33.3333 32.4493 33.3333 33.3333V35" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M20 13.3333H33.3333" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M26.6667 13.3333V8.33333C26.6667 7.44928 27.0179 6.60143 27.643 5.97631C28.2681 5.35119 29.116 5 30 5" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M26.6667 22.5C27.1269 22.5 27.5 22.1269 27.5 21.6667C27.5 21.2064 27.1269 20.8333 26.6667 20.8333C26.2064 20.8333 25.8333 21.2064 25.8333 21.6667C25.8333 22.1269 26.2064 22.5 26.6667 22.5Z" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M30 5.83333C30.4602 5.83333 30.8333 5.46024 30.8333 5C30.8333 4.53976 30.4602 4.16667 30 4.16667C29.5398 4.16667 29.1667 4.53976 29.1667 5C29.1667 5.46024 29.5398 5.83333 30 5.83333Z" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M33.3333 35.8333C33.7936 35.8333 34.1667 35.4602 34.1667 35C34.1667 34.5398 33.7936 34.1667 33.3333 34.1667C32.8731 34.1667 32.5 34.5398 32.5 35C32.5 35.4602 32.8731 35.8333 33.3333 35.8333Z" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M33.3333 14.1667C33.7936 14.1667 34.1667 13.7936 34.1667 13.3333C34.1667 12.8731 33.7936 12.5 33.3333 12.5C32.8731 12.5 32.5 12.8731 32.5 13.3333C32.5 13.7936 32.8731 14.1667 33.3333 14.1667Z" stroke="#155DFC" strokeWidth="3.33333" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        </div>

        <div className="mt-8 flex flex-col items-center gap-2 text-center w-full">
          <h1 className="text-2xl font-semibold text-[#18181b] dark:text-[#fafafa] tracking-[-0.53px] leading-8">
            扫描与生成
          </h1>
          <div className="flex items-center h-5 px-[2px]">
            <p className="text-sm leading-5 text-[#71717b] dark:text-[#a1a1aa] tracking-[-0.15px]">
              正在分析已安装应用并预测最佳输入法规则...
            </p>
          </div>
        </div>

        <div className="mt-8 flex flex-col gap-3 w-full">
          <div className="w-full rounded-full bg-[#e4e4e7] dark:bg-zinc-800 overflow-hidden h-2">
            <motion.div
              initial={{ width: 0 }}
              animate={{ width: `${progress}%` }}
              transition={{ duration: 0.6, ease: "easeOut" }}
              className="h-full bg-[#155dfc]"
            />
          </div>
          <div className="flex items-center justify-between text-xs text-[#9f9fa9] h-4">
            <span className="tracking-[-0.1px]">{statusText}</span>
            <span className="tracking-[-0.1px] tabular-nums">{progress}%</span>
          </div>
          {phase === "error" && (
            <div className="text-xs text-red-500 text-center">{errorMessage}</div>
          )}
        </div>

        <div className="mt-8 w-full">
          <Button
            className={cn(
              "w-full h-[68px] rounded-[10px]",
              "bg-[#009966] hover:bg-[#008055]",
              "text-white text-sm font-medium tracking-[-0.15px]",
              "flex items-center justify-between px-[63px]",
              phase !== "generated" && "opacity-50 cursor-not-allowed"
            )}
            onClick={() => {
              if (phase === "generated") {
                router.push("/");
              }
            }}
            disabled={phase !== "generated"}
          >
            <span>开启 SmartIME之旅</span>
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path d="M7.99999 14.6667C11.6819 14.6667 14.6667 11.6819 14.6667 8.00001C14.6667 4.31811 11.6819 1.33334 7.99999 1.33334C4.3181 1.33334 1.33333 4.31811 1.33333 8.00001C1.33333 11.6819 4.3181 14.6667 7.99999 14.6667Z" stroke="white" strokeWidth="1.33333" strokeLinecap="round" strokeLinejoin="round" />
              <path d="M6 7.99999L7.33333 9.33332L10 6.66666" stroke="white" strokeWidth="1.33333" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </Button>
        </div>
      </motion.div>
    </div>
  );
}
