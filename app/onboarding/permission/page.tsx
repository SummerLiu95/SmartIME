"use client";

import React, { useState, useEffect } from "react";
import { Shield, Settings, ChevronRight, CheckCircle, AlertCircle } from "lucide-react";
import { motion } from "framer-motion";
import { API } from "@/lib/api";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useRouter } from "next/navigation";

export default function OnboardingPage() {
  const router = useRouter();
  const [permissionStatus, setPermissionStatus] = useState<
    "idle" | "checking" | "granted" | "denied"
  >("idle");
  const [isHoveringGuide, setIsHoveringGuide] = useState(false);

  useEffect(() => {
    // 初始静默检查
    checkPermission(true);
  }, []);

  const checkPermission = async (silent = false) => {
    if (!silent) setPermissionStatus("checking");
    
    try {
      const granted = await API.checkPermissions();

      if (granted) {
        setPermissionStatus("granted");
      } else {
        setPermissionStatus("denied");
      }
    } catch (error) {
      console.error("Failed to check permissions:", error);
      setPermissionStatus("denied");
    }
  };

  const handleTriggerPermissionPrompt = async () => {
    try {
      await API.requestPermissions();
    } catch (error) {
      console.error("Failed to request permissions:", error);
    }
  };

  return (
    <div className="w-full bg-white dark:bg-zinc-900 flex justify-center">
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.5 }}
        className={cn(
          "relative flex flex-col items-center w-full max-w-[384px]",
          "px-8 pt-8 pb-24" // 32px padding top/x, 96px padding bottom as per Figma
        )}
      >
        {/* Header Icon */}
        <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-[16px] bg-[#dbeafe] dark:bg-blue-900/30">
          <Shield className="h-8 w-8 text-blue-600 dark:text-blue-400" />
        </div>

        {/* Title & Description */}
        <div className="mt-8 flex flex-col items-center gap-2 text-center w-full">
          <h1 className="text-2xl font-semibold text-[#18181b] dark:text-[#fafafa] tracking-[-0.53px] leading-8">
            权限授予
          </h1>
          <p className="text-sm leading-5 text-[#71717b] dark:text-[#a1a1aa] px-1 tracking-[-0.15px]">
            SmartIME 需要辅助功能权限来监听应用切换并自动调整输入法。
          </p>
        </div>

        {/* Step Guide Card */}
        <div
          className={cn(
            "mt-6 flex w-full flex-col justify-between",
            "rounded-[14px] border border-[#e4e4e7] dark:border-zinc-800",
            "bg-[#fafafa] dark:bg-zinc-800/50",
            "p-4 pr-11 shadow-[0px_1px_3px_0px_rgba(0,0,0,0.1),0px_1px_2px_-1px_rgba(0,0,0,0.1)]",
            "transition-colors cursor-pointer",
            "hover:border-blue-200 dark:hover:border-blue-800",
            "relative" // For absolute positioning of ChevronRight
          )}
          onClick={handleTriggerPermissionPrompt}
          onMouseEnter={() => setIsHoveringGuide(true)}
          onMouseLeave={() => setIsHoveringGuide(false)}
        >
          {/* Step 1: Settings Path */}
          <div className="flex items-start gap-3">
             <div className="mt-1 flex h-6 w-6 shrink-0 items-center justify-center rounded bg-[#e4e4e7] dark:bg-zinc-700">
                <Settings className="h-4 w-4 text-zinc-600 dark:text-zinc-300" />
             </div>
             <div className="flex flex-col gap-1">
                <div className="flex items-center h-5">
                    <span className="text-sm font-medium text-[#18181b] dark:text-[#fafafa] leading-5 tracking-[-0.15px]">
                        设置 &gt; 隐私与安全性 &gt; 辅助功能
                    </span>
                </div>
                <div className="flex items-center h-5">
                    <span className="text-sm text-[#71717b] dark:text-[#a1a1aa] leading-5 tracking-[-0.15px]">
                        点击此卡片触发系统授权弹窗。
                    </span>
                </div>
             </div>
          </div>
          
          {/* External Link Icon Hint */}
          <motion.div 
             className="absolute right-4 top-1/2 -translate-y-1/2 text-zinc-400"
             animate={{ x: isHoveringGuide ? 3 : 0 }}
          >
             <ChevronRight className="h-5 w-5" />
          </motion.div>
        </div>

        {/* Action Button */}
        <div className="mt-6 w-full">
          <Button
            className={cn(
              "w-full h-[68px] rounded-[10px]",
              "bg-[#155dfc] hover:bg-[#155dfc]/90",
              "text-white text-sm font-medium tracking-[-0.15px]",
              "flex items-center justify-between px-8 py-[25px]",
              permissionStatus === "granted" && "bg-green-600 hover:bg-green-700",
              permissionStatus === "denied" && "bg-red-600 hover:bg-red-700"
            )}
            onClick={() => {
              if (permissionStatus === "granted") {
                router.push("/onboarding/llm");
              } else {
                checkPermission(false);
              }
            }}
            disabled={permissionStatus === "checking"}
          >
            {permissionStatus === "checking" ? (
              <span className="flex items-center gap-2">
                <motion.div
                  animate={{ rotate: 360 }}
                  transition={{ repeat: Infinity, duration: 1, ease: "linear" }}
                  className="h-4 w-4 border-2 border-white border-t-transparent rounded-full"
                />
                检查中...
              </span>
            ) : permissionStatus === "granted" ? (
              <span className="flex items-center gap-2">
                 <CheckCircle className="h-4 w-4" />
                 已获得授权
              </span>
            ) : permissionStatus === "denied" ? (
              <span className="flex items-center gap-2">
                 <AlertCircle className="h-4 w-4" />
                 未检测到权限，点击重试
              </span>
            ) : (
              <span>我已开启，继续</span>
            )}
            
            {permissionStatus !== "checking" && (
                <ChevronRight className="h-4 w-4 opacity-50" />
            )}
          </Button>
        </div>
      </motion.div>
    </div>
  );
}
