import '@testing-library/jest-dom/vitest'
import { cleanup } from '@testing-library/react'
import { afterEach, vi } from 'vitest'

// Mock localStorage
const localStorageMock = (() => {
    let store: Record<string, string> = {}
    return {
        getItem: (key: string) => store[key] || null,
        setItem: (key: string, value: string) => { store[key] = value },
        removeItem: (key: string) => { delete store[key] },
        clear: () => { store = {} },
        get length() { return Object.keys(store).length },
        key: (i: number) => Object.keys(store)[i] || null,
    }
})()
Object.defineProperty(window, 'localStorage', { value: localStorageMock })

// Cleanup after each test
afterEach(() => {
    cleanup()
    localStorageMock.clear()
})

// Mock Tauri API globally
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}))


vi.mock('@tauri-apps/plugin-store', () => ({
    Store: vi.fn().mockImplementation(() => ({
        get: vi.fn(),
        set: vi.fn(),
        save: vi.fn(),
        delete: vi.fn(),
    })),
}))

vi.mock('@tauri-apps/plugin-notification', () => ({
    sendNotification: vi.fn(),
    requestPermission: vi.fn().mockResolvedValue('granted'),
    isPermissionGranted: vi.fn().mockResolvedValue(true),
}))
