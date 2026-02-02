import * as React from 'react'

import { HeadContent, Scripts, createRootRoute } from '@tanstack/react-router'

import appCss from '../styles.css?url'
import { tauri } from '@/lib/tauri'

export const Route = createRootRoute({
  head: () => ({
    meta: [
      {
        charSet: 'utf-8',
      },
      {
        name: 'viewport',
        content: 'width=device-width, initial-scale=1',
      },
      {
        title: 'Life OS',
      },
    ],
    links: [
      {
        rel: 'stylesheet',
        href: appCss,
      },
    ],
  }),

  shellComponent: RootDocument,
})

function RootDocument({ children }: { children: React.ReactNode }) {
  React.useEffect(() => {
    const key = 'lifeos:lastExerciseSyncAt'
    const now = Date.now()

    const lastRaw = window.localStorage.getItem(key)
    const last = lastRaw ? Number(lastRaw) : 0
    const twentyFourHoursMs = 24 * 60 * 60 * 1000

    const shouldSync = !Number.isFinite(last) || now - last >= twentyFourHoursMs
    if (!shouldSync) return

    // Sync exercises in background (best-effort)
    tauri
      .fetchAndCacheExercises()
      .then(() => {
        window.localStorage.setItem(key, String(now))
      })
      .catch((err) => {
        console.error('Failed to sync exercises:', err)
      })
  }, [])

  React.useEffect(() => {
    let intervalId: number | undefined

    const syncGoogle = async () => {
      try {
        const status = await tauri.getGoogleSyncStatus()
        if (!status.connected) return
        await tauri.googleSyncNow()
      } catch (err) {
        console.warn('Google sync failed:', err)
      }
    }

    syncGoogle()
    intervalId = window.setInterval(syncGoogle, 10 * 60 * 1000)

    return () => {
      if (intervalId) window.clearInterval(intervalId)
    }
  }, [])
  return (
    <html lang="en" className="h-full" suppressHydrationWarning>
      <head>
        <HeadContent />
      </head>
      <body className="min-h-screen bg-background text-foreground">
        {children}
        <Scripts />
      </body>
    </html>
  )
}
