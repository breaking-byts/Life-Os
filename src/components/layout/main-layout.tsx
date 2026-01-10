import { Link, Outlet, useRouterState } from '@tanstack/react-router'
import {
  BookOpenIcon,
  CalendarRangeIcon,
  DumbbellIcon,
  MenuIcon,
  SettingsIcon,
  SparklesIcon,
  SquareStackIcon,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet'
import { cn } from '@/lib/utils'

const navItems = [
  { label: 'Dashboard', to: '/dashboard', icon: SquareStackIcon },
  { label: 'Academic', to: '/academic', icon: BookOpenIcon },
  { label: 'Skills', to: '/skills', icon: SparklesIcon },
  { label: 'Physical', to: '/physical', icon: DumbbellIcon },
  { label: 'Weekly', to: '/weekly', icon: CalendarRangeIcon },
  { label: 'Settings', to: '/settings', icon: SettingsIcon },
]

function NavLinks({ onNavigate }: { onNavigate?: () => void }) {
  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  })

  return (
    <nav className="space-y-1">
      {navItems.map((item) => {
        const isActive =
          pathname === item.to ||
          (item.to !== '/' && pathname.startsWith(item.to))
        const Icon = item.icon

        return (
          <Link
            key={item.to}
            to={item.to}
            onClick={onNavigate}
            className={cn(
              'hover:bg-muted flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors',
              isActive && 'bg-muted text-primary',
            )}
          >
            <Icon className="h-4 w-4" />
            <span>{item.label}</span>
          </Link>
        )
      })}
    </nav>
  )
}

function Sidebar() {
  return (
    <aside className="border-r bg-muted/40 hidden w-60 shrink-0 flex-col p-4 md:flex">
      <div className="mb-6 space-y-1">
        <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          Life OS
        </p>
        <p className="text-lg font-semibold">Life OS</p>
      </div>
      <ScrollArea className="flex-1">
        <NavLinks />
      </ScrollArea>
      <Separator className="my-4" />
    </aside>
  )
}

function MobileNav() {
  return (
    <Sheet>
      <SheetTrigger asChild>
        <Button variant="ghost" size="icon" className="md:hidden">
          <MenuIcon className="h-5 w-5" />
          <span className="sr-only">Open navigation</span>
        </Button>
      </SheetTrigger>
      <SheetContent side="left" className="w-64 p-4">
        <div className="mb-4 space-y-1">
          <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            Life OS
          </p>
          <p className="text-lg font-semibold">Life OS</p>
        </div>
        <NavLinks onNavigate={() => {}} />
      </SheetContent>
    </Sheet>
  )
}

export function MainLayout({ children }: { children?: React.ReactNode }) {
  return (
    <div className="bg-background text-foreground flex min-h-screen">
      <Sidebar />
      <div className="flex w-full flex-1 flex-col">
        <header className="border-b bg-background/60 sticky top-0 z-10 flex items-center justify-between px-4 py-3 backdrop-blur">
          <div className="flex items-center gap-2">
            <MobileNav />
            <div className="hidden flex-col md:flex">
              <span className="text-xs text-muted-foreground">Life OS</span>
              <span className="text-sm font-semibold">Life OS</span>
            </div>
          </div>
        </header>
        <main className="flex-1 px-4 py-6 md:px-8">
          {children ?? <Outlet />}
        </main>
      </div>
    </div>
  )
}
