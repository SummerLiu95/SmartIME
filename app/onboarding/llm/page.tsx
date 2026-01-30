"use client";

import React, { useState, useEffect } from "react";
import { BrainCircuit, ChevronRight, CheckCircle, AlertCircle, Loader2, Eye, EyeOff } from "lucide-react";
import { motion } from "framer-motion";
import { API, LLMConfig } from "@/lib/api";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useRouter } from "next/navigation";

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
      // Only update if we got valid values (not empty)
      // Note: api_key might be masked "******", handle that if needed, 
      // but for initial onboarding it might be empty.
      if (savedConfig.base_url) {
          setConfig(prev => ({
              ...prev,
              model: savedConfig.model || prev.model,
              base_url: savedConfig.base_url || prev.base_url,
              // Don't overwrite api_key with masked value if we want user to input it
              // But if it's saved, maybe we want to show it's set?
              // For onboarding, usually we want fresh input or show placeholder.
              // If it's masked, we clear it so user enters new one.
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
      router.push("/onboarding/scan"); // Next step (to be implemented)
    } catch (e) {
      console.error("Failed to save", e);
    }
  };

  return (
    <div className="w-full bg-white dark:bg-zinc-900">
      <motion.div
        initial={{ opacity: 0, x: 20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.5 }}
        className="relative flex flex-col items-center min-h-fit w-full max-w-[520px] mx-auto p-8 pb-8"
      >
        {/* Header Icon */}
        <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-purple-100 dark:bg-purple-900/30">
          <BrainCircuit className="h-8 w-8 text-purple-600 dark:text-purple-400" />
        </div>

        {/* Title */}
        <div className="mt-8 flex flex-col items-center gap-2 text-center">
          <h1 className="text-2xl font-semibold text-zinc-900 dark:text-zinc-50">
            配置 AI 模型
          </h1>
          <p className="text-sm leading-5 text-zinc-500 dark:text-zinc-400 px-4">
            SmartIME 使用 AI 来理解您的应用场景。请配置 OpenAI 兼容的 API 服务。
          </p>
        </div>

        {/* Form */}
        <div className="mt-8 w-full max-w-[320px] flex flex-col gap-4">
          <div className="space-y-2">
            <Label htmlFor="base_url">API Base URL</Label>
            <Input
              id="base_url"
              value={config.base_url}
              onChange={(e) => setConfig({ ...config, base_url: e.target.value })}
              placeholder="https://api.openai.com/v1"
              className="bg-zinc-50 dark:bg-zinc-800/50"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="api_key">API Key</Label>
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
                className="bg-zinc-50 dark:bg-zinc-800/50 pr-10"
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

          <div className="space-y-2">
            <Label htmlFor="model">Model Name</Label>
            <Input
                id="model"
                value={config.model}
                onChange={(e) => setConfig({ ...config, model: e.target.value })}
                placeholder="e.g. gpt-4o-mini"
                className="bg-zinc-50 dark:bg-zinc-800/50"
            />
          </div>

          {/* Status Message */}
          <div className="h-6 flex items-center justify-center text-xs">
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
            variant="outline"
            onClick={handleTestConnection}
            disabled={status === "testing" || !config.api_key}
            className="w-full"
          >
            测试连接
          </Button>
        </div>

        {/* Action Button */}
        <div className="mt-auto w-full">
          <Button
            className={cn(
              "w-full h-[68px] rounded-[10px]",
              "text-white text-sm font-medium",
              "flex items-center justify-between px-8",
              status === "success" 
                ? "bg-[#155dfc] hover:bg-[#155dfc]/90"
                : "bg-zinc-200 text-zinc-400 hover:bg-zinc-200 cursor-not-allowed dark:bg-zinc-800 dark:text-zinc-600"
            )}
            onClick={handleSaveAndContinue}
            disabled={status !== "success"}
          >
            <span>保存并继续</span>
            <ChevronRight className="h-4 w-4 opacity-50" />
          </Button>
        </div>
      </motion.div>
    </div>
  );
}
