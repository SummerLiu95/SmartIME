"use client"

import type { HTMLAttributes } from "react"
import Link from "next/link"
import { usePathname } from "next/navigation"
import { cn } from "@/lib/utils"
import { LayoutGrid, Settings } from "lucide-react"

type SidebarProps = HTMLAttributes<HTMLDivElement>

export function Sidebar({ className }: SidebarProps) {
  const pathname = usePathname()

  const routes = [
    {
      label: "规则管理",
      icon: LayoutGrid,
      href: "/rules",
      active: pathname === "/rules" || pathname === "/",
    },
    {
      label: "常规设置",
      icon: Settings,
      href: "/settings/general",
      active: pathname === "/settings/general",
    },
  ]

  return (
    <div className={cn("w-64 border-r border-[#e4e4e7] dark:border-zinc-800 bg-[#fafafa]/50 dark:bg-zinc-900/50 flex flex-col h-screen", className)}>
      <div className="flex-1 px-6 pt-8 pb-4">
        {/* Header */}
        <div className="flex items-center gap-[10px] mb-8 px-0">
          <div className="flex items-center justify-center w-8 h-8 rounded-[10px] bg-[#155dfc] shadow-[0px_4px_6px_-4px_rgba(43,127,255,0.2),0px_10px_15px_-3px_rgba(43,127,255,0.2)]">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M8 3.33334V12.6667" stroke="white" strokeWidth="1.33333" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M3.33331 8H12.6666" stroke="white" strokeWidth="1.33333" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </div>
          <span className="text-base font-bold text-[#18181b] dark:text-[#fafafa] tracking-[-0.71px]">
            SmartIME
          </span>
        </div>

        {/* Navigation */}
        <div className="space-y-1">
          {routes.map((route) => (
            <Link
              key={route.href}
              href={route.href}
              className={cn(
                "flex items-center justify-between w-full h-9 px-3 py-2 rounded-[10px] transition-all duration-200",
                route.active 
                  ? "bg-white dark:bg-zinc-800 shadow-[0px_1px_2px_-1px_rgba(0,0,0,0.1),0px_1px_3px_0px_rgba(0,0,0,0.1)]" 
                  : "hover:bg-black/5 dark:hover:bg-white/5"
              )}
            >
              <div className="flex items-center gap-2">
                <route.icon 
                  className={cn(
                    "h-4 w-4",
                    route.active ? "text-[#155dfc]" : "text-[#71717b] dark:text-[#a1a1aa]"
                  )} 
                />
                <span 
                  className={cn(
                    "text-sm font-medium tracking-[-0.15px]",
                    route.active ? "text-[#155dfc]" : "text-[#71717b] dark:text-[#a1a1aa]"
                  )}
                >
                  {route.label}
                </span>
              </div>
            </Link>
          ))}
        </div>
      </div>

      {/* Status Footer */}
      <div className="mt-auto px-4 pb-6">
        <div className="rounded-[14px] border border-[#dbeafe]/50 bg-[#eff6ff]/50 dark:bg-blue-950/20 dark:border-blue-900/30 px-3 py-3">
          <div className="text-[10px] font-bold text-[#155dfc]/80 dark:text-blue-400 tracking-[1.12px] uppercase mb-1">
            Status
          </div>
          <div className="text-xs font-medium text-[#52525c] dark:text-[#a1a1aa]">
            AI 预测已启用
          </div>
        </div>
      </div>
    </div>
  )
}
