"use client";

import React, { useEffect, useMemo, useRef, useState } from "react";
import Image from "next/image";
import AppLayout from "@/components/layout/app-layout";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import {
  API,
  AppConfig,
  AppIconMap,
  InputSource,
  RuleScanProgress,
} from "@/lib/api";
import { cn } from "@/lib/utils";
import { Search, Trash2 } from "lucide-react";
import { InputMethodSelector } from "@/components/settings/rules/input-method-selector";

const EMPTY_CONFIG: AppConfig = {
  version: 1,
  global_switch: true,
  default_input: "keep",
  general: {
    auto_start: false,
    hide_dock_icon: false,
  },
  rules: [],
};

const FIRST_SCREEN_ICON_BATCH_SIZE = 8;
const BACKGROUND_ICON_BATCH_SIZE = 24;

type AppIconStatus = "pending" | "resolved" | "missing";
type AppIconStatusMap = Record<string, AppIconStatus>;

const pruneRecord = <T extends string>(
  record: Record<string, T>,
  keep: Set<string>
): Record<string, T> => {
  let changed = false;
  const next: Record<string, T> = {};

  for (const [key, value] of Object.entries(record)) {
    if (!keep.has(key)) {
      changed = true;
      continue;
    }
    next[key] = value;
  }

  return changed ? next : record;
};

const chunkBundleIds = (bundleIds: string[], size: number): string[][] => {
  const chunks: string[][] = [];
  for (let index = 0; index < bundleIds.length; index += size) {
    chunks.push(bundleIds.slice(index, index + size));
  }
  return chunks;
};

export default function RulesPage() {
  const [config, setConfig] = useState<AppConfig>(EMPTY_CONFIG);
  const [inputSources, setInputSources] = useState<InputSource[]>([]);
  const [search, setSearch] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isRescanning, setIsRescanning] = useState(false);
  const [appVersion, setAppVersion] = useState<string>("");
  const [appIcons, setAppIcons] = useState<AppIconMap>({});
  const [appIconStatus, setAppIconStatus] = useState<AppIconStatusMap>({});
  const [iconReloadToken, setIconReloadToken] = useState(0);
  const [scanProgress, setScanProgress] = useState<RuleScanProgress | null>(null);
  const isMountedRef = useRef(false);
  const iconRequestIdRef = useRef(0);
  const lastIconReloadTokenRef = useRef(0);
  const appIconsRef = useRef<AppIconMap>({});
  const appIconStatusRef = useRef<AppIconStatusMap>({});

  useEffect(() => {
    isMountedRef.current = true;
    let unlistenProgress: (() => void) | undefined;
    API.onRuleScanProgress((nextProgress) => {
      if (isMountedRef.current) {
        setScanProgress(nextProgress);
      }
    }).then((unlisten) => {
      if (isMountedRef.current) {
        unlistenProgress = unlisten;
      } else {
        unlisten();
      }
    });
    return () => {
      isMountedRef.current = false;
      unlistenProgress?.();
    };
  }, []);

  useEffect(() => {
    appIconsRef.current = appIcons;
  }, [appIcons]);

  useEffect(() => {
    appIconStatusRef.current = appIconStatus;
  }, [appIconStatus]);

  useEffect(() => {
    const load = async () => {
      try {
        const [currentConfig, sources, rescanning] = await Promise.all([
          API.getConfig(),
          API.getSystemInputSources(),
          API.isRescanning(),
        ]);
        if (!isMountedRef.current) return;
        setConfig(currentConfig);
        setInputSources(sources);
        setIsRescanning(rescanning);
      } catch (error) {
        console.error("Failed to load rules data", error);
      } finally {
        if (isMountedRef.current) {
          setIsLoading(false);
        }
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
        if (isMountedRef.current) {
          setAppVersion(version);
        }
      } catch (error) {
        console.error("Failed to get app version", error);
      }
    };

    loadVersion();
  }, []);

  useEffect(() => {
    if (!isRescanning) return;

    const timer = setInterval(async () => {
      try {
        const rescanning = await API.isRescanning();
        if (!isMountedRef.current) return;
        if (rescanning) return;

        setIsRescanning(false);

        const [currentConfig, sources] = await Promise.all([
          API.getConfig(),
          API.getSystemInputSources(),
        ]);
        if (!isMountedRef.current) return;
        setConfig(currentConfig);
        setInputSources(sources);
        setIconReloadToken((token) => token + 1);
      } catch (error) {
        console.error("Failed to sync rescan state", error);
      }
    }, 1200);

    return () => clearInterval(timer);
  }, [isRescanning]);

  const rules = useMemo(() => config.rules ?? [], [config]);

  const ruleBundleKey = useMemo(
    () => rules.map((rule) => rule.bundle_id).join("\n"),
    [rules]
  );

  const filteredRules = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) return rules;
    return rules.filter((rule) =>
      rule.app_name.toLowerCase().includes(keyword) ||
      rule.bundle_id.toLowerCase().includes(keyword)
    );
  }, [rules, search]);

  const visibleRuleBundleIds = useMemo(() => {
    const seen = new Set<string>();
    return filteredRules
      .map((rule) => rule.bundle_id.trim())
      .filter((bundleId) => {
        if (!bundleId || seen.has(bundleId)) return false;
        seen.add(bundleId);
        return true;
      });
  }, [filteredRules]);

  const visibleRuleBundleKey = useMemo(
    () => visibleRuleBundleIds.join("\n"),
    [visibleRuleBundleIds]
  );

  useEffect(() => {
    if (!ruleBundleKey) {
      setAppIcons({});
      setAppIconStatus({});
      return;
    }

    const activeBundleIds = new Set(rules.map((rule) => rule.bundle_id));
    setAppIcons((prev) => pruneRecord(prev, activeBundleIds));
    setAppIconStatus((prev) => pruneRecord(prev, activeBundleIds));
  }, [ruleBundleKey, rules]);

  useEffect(() => {
    if (!visibleRuleBundleKey) return;

    const requestId = iconRequestIdRef.current + 1;
    iconRequestIdRef.current = requestId;
    const forceReload = lastIconReloadTokenRef.current !== iconReloadToken;
    lastIconReloadTokenRef.current = iconReloadToken;

    const bundleIdsToLoad = visibleRuleBundleIds.filter((bundleId) => {
      if (forceReload) return true;
      return (
        !appIconsRef.current[bundleId] &&
        appIconStatusRef.current[bundleId] !== "missing"
      );
    });

    if (bundleIdsToLoad.length === 0) {
      return;
    }

    const firstBatch = bundleIdsToLoad.slice(0, FIRST_SCREEN_ICON_BATCH_SIZE);
    const backgroundBatches = chunkBundleIds(
      bundleIdsToLoad.slice(FIRST_SCREEN_ICON_BATCH_SIZE),
      BACKGROUND_ICON_BATCH_SIZE
    );

    const markPending = (bundleIds: string[]) => {
      setAppIconStatus((prev) => {
        const next = { ...prev };
        for (const bundleId of bundleIds) {
          if (!appIconsRef.current[bundleId]) {
            next[bundleId] = "pending";
          }
        }
        return next;
      });
    };

    const applyBatch = async (bundleIds: string[]) => {
      if (bundleIds.length === 0) return;
      markPending(bundleIds);

      try {
        const icons = await API.getAppIcons(bundleIds);
        if (!isMountedRef.current || iconRequestIdRef.current !== requestId) return;

        setAppIcons((prev) => {
          const next = { ...prev, ...icons };
          for (const bundleId of bundleIds) {
            if (!icons[bundleId]) {
              delete next[bundleId];
            }
          }
          return next;
        });
        setAppIconStatus((prev) => {
          const next = { ...prev };
          for (const bundleId of bundleIds) {
            next[bundleId] = icons[bundleId] ? "resolved" : "missing";
          }
          return next;
        });
      } catch (error) {
        console.error("Failed to load app icons", error);
        if (!isMountedRef.current || iconRequestIdRef.current !== requestId) return;
        setAppIconStatus((prev) => {
          const next = { ...prev };
          for (const bundleId of bundleIds) {
            if (!appIconsRef.current[bundleId]) {
              next[bundleId] = "missing";
            }
          }
          return next;
        });
      }
    };

    const loadBatches = async () => {
      await applyBatch(firstBatch);

      for (const batch of backgroundBatches) {
        if (!isMountedRef.current || iconRequestIdRef.current !== requestId) return;
        await applyBatch(batch);
        if (!isMountedRef.current || iconRequestIdRef.current !== requestId) return;
        await new Promise((resolve) => setTimeout(resolve, 0));
      }
    };

    loadBatches();
  }, [visibleRuleBundleIds, visibleRuleBundleKey, iconReloadToken]);

  const handleSaveRules = async (nextRules: AppConfig["rules"]) => {
    if (isMountedRef.current) {
      setConfig((prev) => ({ ...prev, rules: nextRules }));
    }
    try {
      await API.saveRules(nextRules);
    } catch (error) {
      console.error("Failed to save config", error);
    }
  };

  const handleRuleUpdate = async (bundleId: string, preferredInput: string) => {
    const nextRules = rules.map((rule) => {
      if (rule.bundle_id === bundleId) {
        return {
          ...rule,
          preferred_input: preferredInput,
          is_ai_generated: false, // Mark as manual override
        };
      }
      return rule;
    });
    await handleSaveRules(nextRules);
  };

  const deleteRule = async (bundleId: string) => {
    const nextRules = rules.filter((rule) => rule.bundle_id !== bundleId);
    await handleSaveRules(nextRules);
  };

  const rescanRules = async () => {
    setIsRescanning(true);
    setScanProgress(null);
    try {
      const merged = await API.rescanAndSaveRules();
      if (isMountedRef.current) {
        setConfig((prev) => ({ ...prev, rules: merged }));
        const sources = await API.getSystemInputSources();
        if (isMountedRef.current) {
          setInputSources(sources);
          setIconReloadToken((token) => token + 1);
        }
      }
    } catch (error) {
      console.error("Rescan failed", error);
      const message =
        typeof error === "string"
          ? error
          : error instanceof Error
            ? error.message
            : "";
      if (
        isMountedRef.current &&
        message.includes("already in progress")
      ) {
        setIsRescanning(true);
      }
    } finally {
      if (isMountedRef.current) {
        try {
          const rescanning = await API.isRescanning();
          if (isMountedRef.current) {
            setIsRescanning(rescanning);
          }
        } catch {
          setIsRescanning(false);
        }
      }
    }
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
              "shadow-none",
              "disabled:bg-[#8cb2ff] disabled:text-white/90 disabled:cursor-not-allowed disabled:opacity-100",
              isRescanning && "transition-none"
            )}
          >
            {isRescanning ? (
              <span className="inline-flex items-center">
                {scanProgress && scanProgress.total_apps > 0
                  ? `同步中 ${scanProgress.completed_apps}/${scanProgress.total_apps}`
                  : "同步中"}
                <span className="ml-1 inline-flex items-center gap-1" aria-hidden>
                  <span className="loading-dot" />
                  <span className="loading-dot loading-dot-2" />
                  <span className="loading-dot loading-dot-3" />
                </span>
              </span>
            ) : (
              "重新扫描"
            )}
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
              const iconSrc = appIcons[rule.bundle_id];
              const iconStatus = appIconStatus[rule.bundle_id];
              const showPendingIcon = !iconSrc && iconStatus !== "missing";
              const showFallbackIcon = !iconSrc && iconStatus === "missing";
              return (
                <div
                  key={rule.bundle_id}
                  className="flex items-center border-b border-[#f4f4f5] dark:border-zinc-800/50 h-[73px]"
                >
                  {/* App Icon */}
                  <div className="w-[130px] pl-2">
                    <div className="w-10 h-10 rounded-[14px] bg-white dark:bg-zinc-800 border border-[#e4e4e7] dark:border-zinc-700 flex items-center justify-center overflow-hidden text-xl shadow-[0px_1px_3px_0px_rgba(0,0,0,0.1)]">
                      {showPendingIcon ? (
                        <Skeleton className="h-6 w-6 rounded-[8px] bg-[#e4e4e7] dark:bg-zinc-700" />
                      ) : iconSrc ? (
                        <Image
                          src={iconSrc}
                          alt=""
                          width={40}
                          height={40}
                          unoptimized
                          className="h-full w-full object-cover"
                          draggable={false}
                        />
                      ) : showFallbackIcon ? (
                        rule.app_name.charAt(0).toUpperCase()
                      ) : null}
                    </div>
                  </div>

                  {/* Name */}
                  <div className="w-[154px] text-sm font-medium text-[#18181b] dark:text-[#fafafa]">
                    {rule.app_name}
                  </div>

                  {/* Input Method Badge */}
                  <div className="w-[229px]">
                    <InputMethodSelector
                      value={rule.preferred_input}
                      options={inputSources}
                      onSelect={(val) => handleRuleUpdate(rule.bundle_id, val)}
                    />
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
        <div className="flex items-center justify-between gap-4 px-6 h-8 border-t border-[#f4f4f5] dark:border-zinc-800/50 text-xs text-[#a1a1aa]">
          <span className="min-w-0 truncate">
            {isRescanning
              ? scanProgress?.phase === "scanning_apps"
                ? "正在扫描应用..."
                : scanProgress && scanProgress.total_apps > 0
                  ? `正在生成缺失规则 ${scanProgress.completed_apps}/${scanProgress.total_apps}...`
                  : "正在复用已有规则并生成缺失规则..."
              : `${rules.length} 个受管应用`}
          </span>
          <span>{appVersion ? `v${appVersion}` : "v--"}</span>
        </div>
      </div>
    </AppLayout>
  );
}
