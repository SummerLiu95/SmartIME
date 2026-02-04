"use client";

import React, { useState, useEffect } from "react";
import { ChevronRight, CheckCircle, AlertCircle, Loader2, Eye, EyeOff, Info } from "lucide-react";
import { motion } from "framer-motion";
import { API, LLMConfig } from "@/lib/api";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useRouter } from "next/navigation";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

export default function LLMOnboardingPage() {
  const router = useRouter();
  const [config, setConfig] = useState<LLMConfig>({
    api_key: "",
    model: "gpt-4o-mini",
    base_url: "https://api.openai.com/v1",
  });
  const [showApiKey, setShowApiKey] = useState(false);
  const [status, setStatus] = useState<"idle" | "testing" | "success" | "error">("idle");
  const [errorMsg, setErrorMsg] = useState("");

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const savedConfig = await API.getLLMConfig();
      if (savedConfig.base_url) {
          setConfig(prev => ({
              ...prev,
              model: savedConfig.model || prev.model,
              base_url: savedConfig.base_url || prev.base_url,
              api_key: savedConfig.api_key === "******" ? "" : savedConfig.api_key
          }));
      }
    } catch (e) {
      console.error("Failed to load config", e);
    }
  };

  const handleTestConnection = async () => {
    if (!config.api_key) {
      setErrorMsg("请输入 API Key");
      setStatus("error");
      return;
    }
    
    setStatus("testing");
    setErrorMsg("");
    
    try {
      await API.checkLLMConnection(config);
      setStatus("success");
    } catch (error) {
      console.error("Connection failed", error);
      setStatus("error");
      const message =
        typeof error === "string"
          ? error
          : error instanceof Error
            ? error.message
            : "";
      setErrorMsg(message || "连接失败，请检查配置");
    }
  };

  const handleSaveAndContinue = async () => {
    try {
      await API.saveLLMConfig(config);
      router.push("/onboarding/scan");
    } catch (e) {
      console.error("Failed to save", e);
    }
  };

  return (
    <div className="w-full bg-white dark:bg-zinc-900 flex justify-center">
      <motion.div
        initial={{ opacity: 0, x: 20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.5 }}
        className={cn(
            "relative flex flex-col items-center w-full max-w-[336px]",
            "px-0 pt-8 pb-12"
        )}
      >
        {/* Title */}
        <div className="flex flex-col items-center gap-2 text-center w-full mb-8">
          <h1 className="text-2xl font-semibold text-[#18181b] dark:text-[#fafafa] tracking-[-0.53px] leading-8">
            LLM 设置
          </h1>
          <p className="text-sm leading-5 text-[#71717b] dark:text-[#a1a1aa] tracking-[-0.15px] px-1">
            配置 AI 模型以获得更精准的自动切换建议。
          </p>
        </div>

        {/* Form */}
        <div className="w-full flex flex-col gap-4">
          <div className="space-y-1.5">
            <div className="flex items-center justify-between">
                <Label htmlFor="api_key" className="text-sm font-medium text-[#18181b] dark:text-[#fafafa] tracking-[-0.15px]">API Key *</Label>
                <TooltipProvider>
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Info className="h-3.5 w-3.5 text-zinc-400 cursor-help" />
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>您的 OpenAI 或兼容服务的 API 密钥</p>
                        </TooltipContent>
                    </Tooltip>
                </TooltipProvider>
            </div>
            <div className="relative">
              <Input
                id="api_key"
                type={showApiKey ? "text" : "password"}
                value={config.api_key}
                onChange={(e) => {
                    setConfig({ ...config, api_key: e.target.value });
                    if (status !== 'idle') setStatus('idle');
                }}
                placeholder="sk-..."
                className={cn(
                    "bg-[#fafafa] dark:bg-zinc-800/50 border-[#e4e4e7] dark:border-zinc-700",
                    "h-[38px] rounded-[10px] px-[11px] py-[7px]",
                    "text-sm text-[#18181b] dark:text-[#fafafa] placeholder:text-[#18181b]/50 dark:placeholder:text-[#fafafa]/50",
                    "focus-visible:ring-1 focus-visible:ring-blue-500",
                    "pr-9"
                )}
              />
              <button
                type="button"
                onClick={() => setShowApiKey(!showApiKey)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300"
              >
                {showApiKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
              </button>
            </div>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="model" className="text-sm font-medium text-[#18181b] dark:text-[#fafafa] tracking-[-0.15px]">Model *</Label>
            <Input
                id="model"
                value={config.model}
                onChange={(e) => setConfig({ ...config, model: e.target.value })}
                placeholder="e.g. gpt-4o-mini"
                className={cn(
                    "bg-[#fafafa] dark:bg-zinc-800/50 border-[#e4e4e7] dark:border-zinc-700",
                    "h-[38px] rounded-[10px] px-[11px] py-[7px]",
                    "text-sm text-[#18181b] dark:text-[#fafafa] placeholder:text-[#18181b]/50 dark:placeholder:text-[#fafafa]/50",
                    "focus-visible:ring-1 focus-visible:ring-blue-500"
                )}
            />
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="base_url" className="text-sm font-medium text-[#18181b] dark:text-[#fafafa] tracking-[-0.15px]">Base URL</Label>
            <Input
                id="base_url"
                value={config.base_url}
                onChange={(e) => setConfig({ ...config, base_url: e.target.value })}
                placeholder="https://api.openai.com/v1"
                className={cn(
                    "bg-[#fafafa] dark:bg-zinc-800/50 border-[#e4e4e7] dark:border-zinc-700",
                    "h-[38px] rounded-[10px] px-[11px] py-[7px]",
                    "text-sm text-[#18181b] dark:text-[#fafafa] placeholder:text-[#18181b]/50 dark:placeholder:text-[#fafafa]/50",
                    "focus-visible:ring-1 focus-visible:ring-blue-500"
                )}
            />
          </div>

          {/* Status Message */}
          <div className="min-h-[24px] flex items-center justify-center text-xs mt-2">
            {status === "testing" && (
              <span className="flex items-center gap-1.5 text-zinc-500">
                <Loader2 className="h-3 w-3 animate-spin" />
                正在测试连接...
              </span>
            )}
            {status === "success" && (
              <span className="flex items-center gap-1.5 text-green-600">
                <CheckCircle className="h-3 w-3" />
                连接成功
              </span>
            )}
            {status === "error" && (
              <span className="flex items-center gap-1.5 text-red-600">
                <AlertCircle className="h-3 w-3" />
                {errorMsg}
              </span>
            )}
          </div>

          <Button
            className={cn(
              "w-full rounded-[10px]",
              "text-white text-sm font-medium tracking-[-0.15px]",
              "h-[52px] mt-2", // Height increased to match visual weight better (Figma had padding 24px 16px -> ~68px total, but for "Test Connection" usually smaller. Let's stick to design: padding: 24px 16px results in 68px height if box-sizing border-box and text height included. Wait, Figma says height 68px for the button container. Let's use h-[68px] if it's the main action, but here we have two buttons logic split. The design shows '测试连接' as the main big blue button.)
              // Actually design shows only "测试连接" (Test Connection) in the blue button.
              // Logic wise: user tests connection -> success -> then saves?
              // Or is "测试连接" the only button and it saves automatically?
              // The design shows "测试连接". Let's use that style.
              "h-[68px]",
              status === "success" 
                ? "bg-green-600 hover:bg-green-700" 
                : "bg-[#155dfc] hover:bg-[#155dfc]/90",
              "transition-all duration-200"
            )}
            onClick={status === "success" ? handleSaveAndContinue : handleTestConnection}
            disabled={status === "testing" || !config.api_key}
          >
            {status === "success" ? (
                <span className="flex items-center gap-2">
                    保存并继续 <ChevronRight className="h-4 w-4" />
                </span>
            ) : (
                "测试连接"
            )}
          </Button>
        </div>
      </motion.div>
    </div>
  );
}
