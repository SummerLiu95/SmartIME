"use client";

import React, { useEffect, useState } from "react";
import AppLayout from "@/components/layout/app-layout";
import { Switch } from "@/components/ui/switch";
import { API, AppConfig } from "@/lib/api";
import { cn } from "@/lib/utils";

const EMPTY_CONFIG: AppConfig = {
  version: 1,
  global_switch: true,
  default_input: "keep",
  general: {
    auto_start: true,
    hide_dock_icon: false,
  },
  rules: [],
};

export default function GeneralSettingsPage() {
  const [config, setConfig] = useState<AppConfig>(EMPTY_CONFIG);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    const load = async () => {
      try {
        const saved = await API.getConfig();
        setConfig(saved);
      } catch (error) {
        console.error("Failed to load config", error);
      }
    };

    load();
  }, []);

  const persistConfig = async (nextConfig: AppConfig) => {
    setConfig(nextConfig);
    setIsSaving(true);
    try {
      await API.saveConfig(nextConfig);
    } catch (error) {
      console.error("Failed to save config", error);
    } finally {
      setIsSaving(false);
    }
  };

  const updateSetting = async (key: keyof AppConfig["general"], value: boolean) => {
    const nextConfig = {
      ...config,
      general: {
        ...config.general,
        [key]: value,
      },
    };
    await persistConfig(nextConfig);
  };

  const updateGlobalSwitch = async (value: boolean) => {
    await persistConfig({
      ...config,
      global_switch: value,
    });
  };

  const settings = [
    {
      key: "auto_start" as const,
      title: "登录时自动启动",
      description: "在您进入系统时开启 SmartIME",
      value: config.general.auto_start,
    },
    {
      key: "hide_dock_icon" as const,
      title: "隐藏 Dock 图标",
      description: "关闭主窗口后继续在菜单栏后台运行（推荐）",
      value: config.general.hide_dock_icon,
    },
  ];

  return (
    <AppLayout>
      <div className="flex flex-col h-full bg-white dark:bg-zinc-900 pt-8 px-8">
        {/* Header */}
        <div className="flex flex-col gap-1 mb-8">
          <h1 className="text-lg font-semibold text-[#18181b] dark:text-[#fafafa] tracking-[-0.44px]">
            常规设置
          </h1>
          <p className="text-sm text-[#71717b] dark:text-[#a1a1aa] tracking-[-0.15px]">
            管理应用的基础运行行为。
          </p>
        </div>

        {/* Settings List */}
        <div className="flex flex-col gap-4 max-w-[670px]">
          <div
            className={cn(
              "flex items-center justify-between px-[15px] h-[72px]",
              "bg-[#fafafa] dark:bg-zinc-800",
              "border border-[#e4e4e7] dark:border-zinc-700",
              "rounded-[14px]"
            )}
          >
            <div className="flex flex-col gap-[2px]">
              <div className="text-sm font-medium text-[#18181b] dark:text-[#fafafa] tracking-[-0.15px]">
                自动切换总开关
              </div>
              <div className="text-xs text-[#71717b] dark:text-[#a1a1aa]">
                关闭后 SmartIME 不会自动切换输入法
              </div>
            </div>
            <Switch
              checked={config.global_switch}
              onCheckedChange={updateGlobalSwitch}
              disabled={isSaving}
              className={cn(
                "data-[state=checked]:bg-[#155dfc]",
                isSaving && "opacity-60"
              )}
            />
          </div>

          {settings.map((setting) => (
            <div
              key={setting.key}
              className={cn(
                "flex items-center justify-between px-[15px] h-[72px]",
                "bg-[#fafafa] dark:bg-zinc-800",
                "border border-[#e4e4e7] dark:border-zinc-700",
                "rounded-[14px]"
              )}
            >
              <div className="flex flex-col gap-[2px]">
                <div className="text-sm font-medium text-[#18181b] dark:text-[#fafafa] tracking-[-0.15px]">
                  {setting.title}
                </div>
                <div className="text-xs text-[#71717b] dark:text-[#a1a1aa]">
                  {setting.description}
                </div>
              </div>
              <Switch
                checked={setting.value}
                onCheckedChange={(value) => updateSetting(setting.key, value)}
                disabled={isSaving}
                className={cn(
                  "data-[state=checked]:bg-[#155dfc]",
                  isSaving && "opacity-60"
                )}
              />
            </div>
          ))}
        </div>
      </div>
    </AppLayout>
  );
}
