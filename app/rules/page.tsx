"use client";

import React, { useEffect, useMemo, useState } from "react";
import AppLayout from "@/components/layout/app-layout";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { API, AppConfig, InputSource } from "@/lib/api";
import { cn } from "@/lib/utils";
import { RefreshCw, Search, Trash2 } from "lucide-react";

const EMPTY_CONFIG: AppConfig = {
  version: 1,
  global_switch: true,
  default_input: "keep",
  general: {
    auto_start: true,
    show_menu_bar_status: true,
    hide_dock_icon: false,
  },
  rules: [],
};

export default function RulesPage() {
  const [config, setConfig] = useState<AppConfig>(EMPTY_CONFIG);
  const [inputSources, setInputSources] = useState<InputSource[]>([]);
  const [search, setSearch] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isRescanning, setIsRescanning] = useState(false);
  const [appVersion, setAppVersion] = useState<string>("");

  useEffect(() => {
    const load = async () => {
      try {
        const [currentConfig, sources] = await Promise.all([
          API.getConfig(),
          API.getSystemInputSources(),
        ]);
        setConfig(currentConfig);
        setInputSources(sources);
      } catch (error) {
        console.error("Failed to load rules data", error);
      } finally {
        setIsLoading(false);
      }
    };

    load();
  }, []);

  useEffect(() => {
    const loadVersion = async () => {
      const isTauri =
        typeof window !== "undefined" &&
        Boolean((window as unknown as { __TAURI_INTERNALS__?: boolean }).__TAURI_INTERNALS__);
      if (!isTauri) return;
      try {
        const { getVersion } = await import("@tauri-apps/api/app");
        const version = await getVersion();
        setAppVersion(version);
      } catch (error) {
        console.error("Failed to get app version", error);
      }
    };

    loadVersion();
  }, []);

  const rules = useMemo(() => config.rules ?? [], [config]);

  const filteredRules = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) return rules;
    return rules.filter((rule) =>
      rule.app_name.toLowerCase().includes(keyword) ||
      rule.bundle_id.toLowerCase().includes(keyword)
    );
  }, [rules, search]);

  const handleSaveConfig = async (nextConfig: AppConfig) => {
    setConfig(nextConfig);
    try {
      await API.saveConfig(nextConfig);
    } catch (error) {
      console.error("Failed to save config", error);
    }
  };

  const deleteRule = async (bundleId: string) => {
    const nextRules = rules.filter((rule) => rule.bundle_id !== bundleId);
    await handleSaveConfig({ ...config, rules: nextRules });
  };

  const rescanRules = async () => {
    setIsRescanning(true);
    try {
      const sources = await API.getSystemInputSources();
      setInputSources(sources);
      const generated = await API.scanAndPredict(sources);
      const manualRules = rules.filter((rule) => !rule.is_ai_generated);
      const manualMap = new Map(manualRules.map((rule) => [rule.bundle_id, rule]));

      const merged = generated
        .filter((rule) => !manualMap.has(rule.bundle_id))
        .concat(manualRules);

      await handleSaveConfig({ ...config, rules: merged });
    } catch (error) {
      console.error("Rescan failed", error);
    } finally {
      setIsRescanning(false);
    }
  };

  const getInputSourceLabel = (inputId: string) => {
    const source = inputSources.find((s) => s.id === inputId);
    if (!source) return { icon: "A", name: inputId };
    
    // Simple heuristic for icon
    const isChinese = source.name.includes("Chinese") || source.id.includes("SCIM");
    return {
      icon: isChinese ? "中" : "A",
      name: source.name.replace("Input Method", "").trim()
    };
  };

  return (
    <AppLayout>
      <div className="flex flex-col h-full bg-white dark:bg-zinc-900">
        {/* Top Bar */}
        <div className="flex items-center justify-between px-6 py-6 border-b border-[#e4e4e7] dark:border-zinc-800 bg-[#fafafa]/50 dark:bg-zinc-900/50 h-[87px]">
          <div className="relative">
            <Search className="absolute left-[11px] top-1/2 -translate-y-1/2 h-4 w-4 text-zinc-400" />
            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="搜索应用..."
              className={cn(
                "w-[384px] h-[38px] pl-[35px] pr-4",
                "bg-[#fafafa] dark:bg-zinc-800",
                "border-[#e4e4e7] dark:border-zinc-700",
                "rounded-[10px]",
                "text-sm placeholder:text-[#18181b]/50 dark:placeholder:text-zinc-500",
                "focus-visible:ring-1 focus-visible:ring-[#155dfc]"
              )}
            />
          </div>
          
          <Button
            onClick={rescanRules}
            disabled={isRescanning || isLoading}
            className={cn(
              "h-[36px] px-4 rounded-[10px]",
              "bg-[#155dfc] hover:bg-[#155dfc]/90",
              "text-white text-sm font-medium",
              "shadow-none"
            )}
          >
            <RefreshCw className={cn("mr-2 h-4 w-4", isRescanning && "animate-spin")} />
            重新扫描
          </Button>
        </div>

        {/* Table Header */}
        <div className="flex items-center px-6 pt-6 pb-2 border-b border-[#f4f4f5] dark:border-zinc-800/50">
          <div className="w-[130px] text-xs font-bold text-[#9f9fa9] pl-2">应用</div>
          <div className="w-[154px] text-xs font-bold text-[#9f9fa9]">名称</div>
          <div className="w-[229px] text-xs font-bold text-[#9f9fa9]">偏好输入法</div>
          <div className="w-[64px] text-right text-xs font-bold text-[#9f9fa9] pr-2 whitespace-nowrap">操作</div>
        </div>

        {/* Table Body */}
        <div className="flex-1 overflow-y-auto px-6">
          {isLoading ? (
            <div className="flex items-center justify-center h-32 text-sm text-zinc-500">
              加载中...
            </div>
          ) : filteredRules.length === 0 ? (
            <div className="flex items-center justify-center h-32 text-sm text-zinc-500">
              暂无规则，请点击重新扫描
            </div>
          ) : (
            filteredRules.map((rule) => {
              const inputInfo = getInputSourceLabel(rule.preferred_input);
              return (
                <div 
                  key={rule.bundle_id}
                  className="flex items-center border-b border-[#f4f4f5] dark:border-zinc-800/50 h-[73px]"
                >
                  {/* App Icon */}
                  <div className="w-[130px] pl-2">
                    <div className="w-10 h-10 rounded-[14px] bg-white dark:bg-zinc-800 border border-[#e4e4e7] dark:border-zinc-700 flex items-center justify-center text-xl shadow-[0px_1px_3px_0px_rgba(0,0,0,0.1)]">
                      {/* Placeholder for app icon - in real app we'd fetch icon */}
                      {rule.app_name.charAt(0).toUpperCase()}
                    </div>
                  </div>

                  {/* Name */}
                  <div className="w-[154px] text-sm font-medium text-[#18181b] dark:text-[#fafafa]">
                    {rule.app_name}
                  </div>

                  {/* Input Method Badge */}
                  <div className="w-[229px]">
                    <div className="inline-flex items-center gap-3 px-3 py-1.5 rounded-[10px] border border-[#e4e4e7] dark:border-zinc-700 bg-white dark:bg-zinc-800 min-w-[84px] h-[32px]">
                      <span className={cn(
                        "text-xs font-medium",
                        inputInfo.icon === "中" ? "text-[#2b7fff]" : "text-[#2b7fff]" // Both blue in Figma design
                      )}>
                        {inputInfo.icon}
                      </span>
                      <span className="text-xs font-medium text-[#52525c] dark:text-zinc-400">
                        {inputInfo.name}
                      </span>
                    </div>
                  </div>

                  {/* Action */}
                  <div className="w-[64px] flex justify-end pr-2">
                    <button
                      onClick={() => deleteRule(rule.bundle_id)}
                      className="text-[#9f9fa9] hover:text-red-500 transition-colors p-2"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </div>
                </div>
              );
            })
          )}
        </div>

        {/* Bottom Indicator */}
        <div className="flex items-center justify-between px-6 h-8 border-t border-[#f4f4f5] dark:border-zinc-800/50 text-xs text-[#a1a1aa]">
          <span>{rules.length} 个受管应用</span>
          <div className="flex items-center gap-1">
            <span>{appVersion ? `v${appVersion}` : "v--"}</span>
            <span className="text-[#a1a1aa]">↗</span>
          </div>
        </div>
      </div>
    </AppLayout>
  );
}
