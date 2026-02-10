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
          <img
            src="/app_icon.svg"
            alt="SmartIME"
            className="h-8 w-8"
          />
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

    </div>
  )
}
