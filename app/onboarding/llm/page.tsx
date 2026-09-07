"use client";

import React, { useState, useEffect } from "react";
import { ChevronRight, CheckCircle, AlertCircle, Loader2, Eye, EyeOff, Info } from "lucide-react";
import { motion } from "framer-motion";
import { API, LLMConfig, LLMProvider } from "@/lib/api";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useRouter } from "next/navigation";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

const PROVIDERS: Array<{ value: LLMProvider; label: string; defaultModel: string }> = [
  { value: "deepseek", label: "DeepSeek", defaultModel: "deepseek-v4-pro" },
  { value: "openai", label: "OpenAI", defaultModel: "gpt-4o-mini" },
  { value: "anthropic", label: "Anthropic", defaultModel: "claude-3-5-haiku-latest" },
  { value: "gemini", label: "Google Gemini", defaultModel: "gemini-2.5-flash" },
];

export default function LLMOnboardingPage() {
  const router = useRouter();
  const [config, setConfig] = useState<LLMConfig>({
    api_key: "",
    provider: "deepseek",
    model: "deepseek-v4-pro",
  });
  const [showApiKey, setShowApiKey] = useState(false);
  const [status, setStatus] = useState<"idle" | "testing" | "success" | "error">("idle");
  const [errorMsg, setErrorMsg] = useState("");
  const [hasApiKey, setHasApiKey] = useState(false);
  const [savedProvider, setSavedProvider] = useState<LLMProvider | null>(null);
  const [preview, setPreview] = useState(false);
  const [busy, setBusy] = useState(true);
  const canReuseKey = hasApiKey && config.provider === savedProvider;

  useEffect(() => {
    const isPreview = !API._isTauri();
    setPreview(isPreview);
    if (isPreview) setConfig(prev => ({ ...prev, api_key: 'demo-key' }));
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const savedConfig = await API.getLLMConfig();
      setHasApiKey(savedConfig.has_api_key);
      setSavedProvider(savedConfig.provider);
      setConfig(prev => ({
        ...prev,
        provider: savedConfig.provider,
        model: savedConfig.model || prev.model,
        api_key: API._isTauri() ? "" : "demo-key"
      }));
    } catch (e) {
      setStatus("error");
      setErrorMsg(typeof e === "string" ? e : "读取配置失败，请重试");
    } finally {
      setBusy(false);
    }
  };

  const handleTestConnection = async () => {
    if (!config.api_key && !canReuseKey) {
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
    setBusy(true);
    try {
      await API.saveLLMConfig(config);
      setConfig(prev => ({ ...prev, api_key: "" }));
      router.push("/onboarding/scan");
    } catch (e) {
      setStatus("error");
      setErrorMsg(typeof e === "string" ? e : "保存失败，请重试");
    } finally {
      setBusy(false);
    }
  };

  const handleDeleteKey = async () => {
    setBusy(true);
    try {
      await API.deleteLLMKey();
      setHasApiKey(false);
      setConfig(prev => ({ ...prev, api_key: "" }));
      setStatus("idle");
      setErrorMsg("");
    } catch (error) {
      setStatus("error");
      setErrorMsg(typeof error === "string" ? error : "删除失败，请重试");
    } finally { setBusy(false); }
  };

  return (
    <div className="w-full h-full bg-white dark:bg-zinc-900 flex justify-center">
      <motion.div
        initial={{ opacity: 0, x: 20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.5 }}
        className={cn(
            "relative flex flex-col items-center w-full max-w-[336px]",
            "px-0 pt-6 pb-6"
        )}
      >
        {/* Title */}
        <div className="flex flex-col items-center gap-2 text-center w-full mb-6">
          <h1 className="text-2xl font-semibold text-[#18181b] dark:text-[#fafafa] tracking-[-0.53px] leading-8">
            LLM 设置
          </h1>
          <p className="text-sm leading-5 text-[#71717b] dark:text-[#a1a1aa] tracking-[-0.15px] px-1">
            {preview ? "浏览器演示：使用虚拟密钥，连接测试为模拟结果。" : "配置 AI 模型以获得更精准的自动切换建议。"}
          </p>
        </div>

        {/* Form */}
        <div className="w-full flex flex-col gap-4">
          <div className="space-y-1.5">
            <Label htmlFor="provider" className="text-sm font-medium text-[#18181b] dark:text-[#fafafa] tracking-[-0.15px]">服务商 *</Label>
            <Select
              value={config.provider}
              disabled={busy || status === "testing"}
              onValueChange={(value: LLMProvider) => {
                const provider = PROVIDERS.find((item) => item.value === value);
                setConfig((current) => ({
                  ...current,
                  provider: value,
                  model: provider?.defaultModel ?? current.model,
                  api_key: value === savedProvider ? current.api_key : "",
                }));
                setStatus("idle");
                setErrorMsg("");
              }}
            >
              <SelectTrigger
                id="provider"
                className={cn(
                  "w-full bg-[#fafafa] dark:bg-zinc-800/50 border-[#e4e4e7] dark:border-zinc-700",
                  "h-[38px] rounded-[10px] px-[11px] py-[7px]",
                  "text-sm text-[#18181b] dark:text-[#fafafa]",
                  "focus-visible:ring-1 focus-visible:ring-blue-500"
                )}
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {PROVIDERS.map((provider) => (
                  <SelectItem key={provider.value} value={provider.value}>
                    {provider.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1.5">
            <div className="flex items-center justify-between">
                <Label htmlFor="api_key" className="text-sm font-medium text-[#18181b] dark:text-[#fafafa] tracking-[-0.15px]">API Key *</Label>
                <TooltipProvider>
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Info className="h-3.5 w-3.5 text-zinc-400 cursor-help" />
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>所选服务商提供的 API 密钥</p>
                        </TooltipContent>
                    </Tooltip>
                </TooltipProvider>
            </div>
            <div className="relative">
              <Input
                id="api_key"
                type={showApiKey ? "text" : "password"}
                disabled={preview || busy || status === "testing"}
                autoComplete="off"
                spellCheck={false}
                value={config.api_key}
                onChange={(e) => {
                    setConfig({ ...config, api_key: e.target.value });
                    if (status !== 'idle') setStatus('idle');
                }}
                placeholder={canReuseKey ? "已安全保存，留空使用现有密钥" : "sk-..."}
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
            {hasApiKey && !preview && (
              <button
                type="button"
                className={cn(
                  "inline-flex h-7 w-fit items-center rounded-md border border-red-200 px-2.5",
                  "text-xs font-medium text-red-600 transition-colors",
                  "hover:border-red-300 hover:bg-red-50 hover:text-red-700",
                  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-500 focus-visible:ring-offset-2",
                  "disabled:pointer-events-none disabled:opacity-50",
                  "dark:border-red-900 dark:hover:border-red-800 dark:hover:bg-red-950/40"
                )}
                disabled={busy || status === "testing"}
                onClick={handleDeleteKey}
              >
                删除已保存的密钥
              </button>
            )}
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="model" className="text-sm font-medium text-[#18181b] dark:text-[#fafafa] tracking-[-0.15px]">Model *</Label>
            <Input
                id="model"
                value={config.model}
                disabled={busy || status === "testing"}
                onChange={(e) => { setConfig({ ...config, model: e.target.value }); setStatus("idle"); }}
                placeholder="e.g. gpt-4o-mini"
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
              <div className="flex flex-col items-center gap-1.5">
                <span className="flex items-center gap-1.5 text-red-600">
                  <AlertCircle className="h-3 w-3" />
                  {errorMsg}
                </span>
                <button type="button" className="text-blue-600" disabled={busy}
                  onClick={() => { setBusy(true); void loadConfig(); }}>
                  重新读取配置
                </button>
              </div>
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
            disabled={busy || status === "testing" || (!config.api_key && !canReuseKey)}
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
