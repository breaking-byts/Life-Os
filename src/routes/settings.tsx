import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { MainLayout } from '@/components/layout/main-layout'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { tauri } from '@/lib/tauri'

export const Route = createFileRoute('/settings')({
  component: SettingsPage,
})

function SettingsPage() {
  const queryClient = useQueryClient()
  const [clientId, setClientId] = useState('')
  const [connecting, setConnecting] = useState(false)
  const [connectError, setConnectError] = useState<string | null>(null)

  const syncStatusQuery = useQuery({
    queryKey: ['google-sync-status'],
    queryFn: tauri.getGoogleSyncStatus,
  })

  const saveClientId = useMutation({
    mutationFn: (value: string) => tauri.setGoogleClientId(value),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['google-sync-status'] })
    },
  })

  const syncNow = useMutation({
    mutationFn: tauri.googleSyncNow,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['google-sync-status'] })
      queryClient.invalidateQueries({ queryKey: ['calendar-items'] })
    },
  })

  const disconnect = useMutation({
    mutationFn: tauri.disconnectGoogle,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['google-sync-status'] })
      queryClient.invalidateQueries({ queryKey: ['calendar-items'] })
    },
  })

  const handleSave = async () => {
    const trimmed = clientId.trim()
    if (!trimmed) return
    await saveClientId.mutateAsync(trimmed)
  }

  const pollForCompletion = async () => {
    let attempts = 0
    setConnectError(null)

    const poll = async () => {
      attempts += 1
      try {
        await tauri.googleOauthComplete()
        await tauri.googleSyncNow()
        setConnecting(false)
        queryClient.invalidateQueries({ queryKey: ['google-sync-status'] })
        queryClient.invalidateQueries({ queryKey: ['calendar-items'] })
      } catch (err) {
        const message = String(err)
        const pending =
          message.includes('callback') || message.includes('not received')
        if (pending && attempts < 120) {
          setTimeout(poll, 1000)
        } else {
          setConnecting(false)
          setConnectError(message)
        }
      }
    }

    poll()
  }

  const handleConnect = async () => {
    setConnecting(true)
    setConnectError(null)

    try {
      const trimmed = clientId.trim()
      if (trimmed) {
        await saveClientId.mutateAsync(trimmed)
      }

      const auth = await tauri.googleOauthBegin()
      window.open(auth.auth_url, '_blank', 'noopener,noreferrer')
      await pollForCompletion()
    } catch (err) {
      setConnecting(false)
      setConnectError(String(err))
    }
  }

  const canConnect =
    clientId.trim().length > 0 || syncStatusQuery.data?.client_id_set

  return (
    <MainLayout>
      <div className="space-y-6">
        <div className="space-y-2">
          <h1 className="text-2xl font-semibold">Settings</h1>
          <p className="text-muted-foreground text-sm">
            Connect Google Calendar to keep your plan and schedule in sync.
          </p>
        </div>

        <div className="rounded-xl border bg-card p-6 space-y-4">
          <div className="space-y-1">
            <h2 className="text-lg font-semibold">Calendar Sync</h2>
            <p className="text-sm text-muted-foreground">
              Provide your OAuth client ID, then connect your Google account.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="google-client-id">Google OAuth Client ID</Label>
            <Input
              id="google-client-id"
              placeholder="12345.apps.googleusercontent.com"
              value={clientId}
              onChange={(event) => setClientId(event.target.value)}
            />
          </div>

          <div className="flex flex-wrap gap-2">
            <Button onClick={handleSave} disabled={saveClientId.isPending}>
              Save client ID
            </Button>
            <Button
              variant="outline"
              onClick={handleConnect}
              disabled={connecting || saveClientId.isPending || !canConnect}
            >
              {connecting ? 'Connecting…' : 'Connect Google'}
            </Button>
            <Button
              variant="outline"
              onClick={() => syncNow.mutate()}
              disabled={!syncStatusQuery.data?.connected || syncNow.isPending}
            >
              Sync now
            </Button>
            <Button
              variant="ghost"
              onClick={() => disconnect.mutate()}
              disabled={!syncStatusQuery.data?.connected || disconnect.isPending}
            >
              Disconnect
            </Button>
          </div>

          {connectError && (
            <p className="text-sm text-red-500">{connectError}</p>
          )}

          <div className="rounded-lg border border-dashed p-3 text-sm text-muted-foreground">
            {syncStatusQuery.data?.connected ? (
              <div className="space-y-1">
                <div>
                  Connected{syncStatusQuery.data.email
                    ? ` as ${syncStatusQuery.data.email}`
                    : ''}
                </div>
                <div>
                  Last sync:{' '}
                  {syncStatusQuery.data.last_sync
                    ? new Date(syncStatusQuery.data.last_sync).toLocaleString()
                    : 'Never'}
                </div>
              </div>
            ) : (
              <div>Not connected yet.</div>
            )}
          </div>
        </div>
      </div>
    </MainLayout>
  )
}
