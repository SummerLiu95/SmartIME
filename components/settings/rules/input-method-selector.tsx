"use client";

import React from "react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { InputSource } from "@/lib/api";
import { cn } from "@/lib/utils";

interface InputMethodSelectorProps {
  value: string;
  options: InputSource[];
  onSelect: (value: string) => void;
}

export function InputMethodSelector({
  value,
  options,
  onSelect,
}: InputMethodSelectorProps) {
  const getInputSourceLabel = (inputId: string, sourceName?: string) => {
    // If sourceName is provided (from options), use it directly
    // Otherwise try to find it in options
    const source = options.find((s) => s.id === inputId);
    const name = sourceName || source?.name || inputId;
    
    // Simple heuristic for icon
    const isChinese = name.includes("Chinese") || inputId.includes("SCIM");
    
    return {
      icon: isChinese ? "中" : "A",
      name: name.replace("Input Method", "").trim(),
      isChinese
    };
  };

  const currentInfo = getInputSourceLabel(value);
  const safeOptions =
    options.length > 0
      ? options
      : [
          {
            id: value,
            name: currentInfo.name,
            category: "keyboard",
          } as InputSource,
        ];

  return (
    <DropdownMenu modal={false}>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className={cn(
            "inline-flex items-center gap-3 px-3 py-1.5 rounded-[10px] border border-[#e4e4e7] dark:border-zinc-700 bg-white dark:bg-zinc-800 min-w-[84px] h-[32px] hover:bg-zinc-50 dark:hover:bg-zinc-700/50 transition-colors",
            "outline-none focus-visible:ring-1 focus-visible:ring-[#155dfc] cursor-pointer"
          )}
          aria-label="选择偏好输入法"
        >
          <span
            className={cn(
              "text-xs font-medium",
              currentInfo.isChinese ? "text-[#2b7fff]" : "text-[#2b7fff]"
            )}
          >
            {currentInfo.icon}
          </span>
          <span className="text-xs font-medium text-[#52525c] dark:text-zinc-400 truncate max-w-[120px]">
            {currentInfo.name}
          </span>
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="min-w-[180px]">
        <DropdownMenuRadioGroup value={value} onValueChange={onSelect}>
          {safeOptions.map((option) => {
            const info = getInputSourceLabel(option.id, option.name);
            return (
              <DropdownMenuRadioItem key={option.id} value={option.id} className="py-2">
                <div className="flex items-center gap-3">
                  <span
                    className={cn(
                      "text-xs font-medium w-4 text-center",
                      info.isChinese ? "text-[#2b7fff]" : "text-[#2b7fff]"
                    )}
                  >
                    {info.icon}
                  </span>
                  <span className="text-xs font-medium text-[#52525c] dark:text-zinc-400">
                    {info.name}
                  </span>
                </div>
              </DropdownMenuRadioItem>
            );
          })}
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
