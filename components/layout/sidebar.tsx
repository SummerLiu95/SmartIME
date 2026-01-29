"use client"

import type { HTMLAttributes } from "react"
import Link from "next/link"
import { usePathname } from "next/navigation"
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import { List, Settings, Sparkles, LayoutGrid } from "lucide-react"

type SidebarProps = HTMLAttributes<HTMLDivElement>

export function Sidebar({ className }: SidebarProps) {
  const pathname = usePathname()

  const routes = [
    {
      label: "Rules",
      icon: List,
      href: "/rules",
      active: pathname === "/rules" || pathname === "/",
    },
    {
      label: "LLM Settings",
      icon: Sparkles,
      href: "/settings/llm",
      active: pathname === "/settings/llm",
    },
    {
      label: "General",
      icon: Settings,
      href: "/settings/general",
      active: pathname === "/settings/general",
    },
  ]

  return (
    <div className={cn("pb-12 w-64 border-r bg-muted/10 h-screen", className)}>
      <div className="space-y-4 py-4">
        <div className="px-3 py-2">
          <div className="flex items-center px-4 mb-6">
            <LayoutGrid className="mr-2 h-6 w-6 text-primary" />
            <h2 className="text-lg font-semibold tracking-tight">
              SmartIME
            </h2>
          </div>
          <div className="space-y-1">
            {routes.map((route) => (
              <Button
                key={route.href}
                variant={route.active ? "secondary" : "ghost"}
                className={cn(
                  "w-full justify-start",
                  route.active && "bg-secondary"
                )}
                asChild
              >
                <Link href={route.href}>
                  <route.icon className="mr-2 h-4 w-4" />
                  {route.label}
                </Link>
              </Button>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
