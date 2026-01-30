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
        // 可以在这里处理跳转逻辑，或者让用户点击按钮跳转
        // setTimeout(() => router.push('/next-step'), 1000); 
      } else {
        setPermissionStatus("denied");
      }
    } catch (error) {
      console.error("Failed to check permissions:", error);
      setPermissionStatus("denied");
    }
  };

  const handleOpenSettings = async () => {
    try {
      await API.openSystemSettings();
    } catch (error) {
      console.error("Failed to open settings:", error);
    }
  };

  return (
    <div className="w-full bg-white dark:bg-zinc-900">
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.5 }}
        className={cn(
          "relative flex flex-col items-center min-h-fit w-full max-w-[480px] mx-auto",
          "p-8 pb-8"
        )}
      >
        {/* Header Icon */}
        <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-blue-100 dark:bg-blue-900/30">
          <Shield className="h-8 w-8 text-blue-600 dark:text-blue-400" />
        </div>

        {/* Title & Description */}
        <div className="mt-8 flex flex-col items-center gap-2 text-center">
          <h1 className="text-2xl font-semibold text-zinc-900 dark:text-zinc-50">
            权限授予
          </h1>
          <p className="text-sm leading-5 text-zinc-500 dark:text-zinc-400 px-4">
            SmartIME 需要辅助功能权限来监听应用切换并自动调整输入法。
          </p>
        </div>

        {/* Step Guide */}
        <div
          className={cn(
            "mt-6 flex w-full flex-col justify-between",
            "rounded-[14px] border border-zinc-200 dark:border-zinc-800",
            "bg-zinc-50 dark:bg-zinc-800/50",
            "p-4 pr-11 shadow-sm transition-colors cursor-pointer",
            "hover:border-blue-200 dark:hover:border-blue-800",
            "relative" // For absolute positioning of ChevronRight
          )}
          onClick={handleOpenSettings}
          onMouseEnter={() => setIsHoveringGuide(true)}
          onMouseLeave={() => setIsHoveringGuide(false)}
        >
          {/* Step 1: Settings Path */}
          <div className="flex items-start gap-3">
             <div className="mt-1 flex h-6 items-center justify-center rounded bg-zinc-200 px-1 dark:bg-zinc-700">
                <Settings className="h-4 w-4 text-zinc-600 dark:text-zinc-300" />
             </div>
             <div className="flex flex-col gap-1">
                <div className="flex items-center h-5">
                    <span className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                        设置 &gt; 隐私与安全性 &gt; 辅助功能
                    </span>
                </div>
                <div className="flex items-center h-5">
                    <span className="text-sm text-zinc-500 dark:text-zinc-400">
                        请在列表中找到并勾选 SmartIME。
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
        <div className="mt-auto w-full">
          <Button
            className={cn(
              "w-full h-[68px] rounded-[10px]",
              "bg-[#155dfc] hover:bg-[#155dfc]/90",
              "text-white text-sm font-medium",
              "flex items-center justify-between px-8",
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
