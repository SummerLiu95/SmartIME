import { Sidebar } from "@/components/layout/sidebar"
import { FadeIn } from "@/components/motion/fade-in"

interface AppLayoutProps {
  children: React.ReactNode
}

export default function AppLayout({ children }: AppLayoutProps) {
  return (
    <div className="flex h-screen overflow-hidden bg-background">
      <Sidebar className="hidden md:block" />
      <main className="flex-1 overflow-y-auto">
        <FadeIn className="h-full p-8">
          {children}
        </FadeIn>
      </main>
    </div>
  )
}
