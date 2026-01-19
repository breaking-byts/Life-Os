import React, { ReactElement } from 'react'
import { render, RenderOptions } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

/**
 * Create a fresh QueryClient for each test
 */
function createTestQueryClient() {
    return new QueryClient({
        defaultOptions: {
            queries: {
                retry: false,
                gcTime: 0,
                staleTime: 0,
            },
            mutations: {
                retry: false,
            },
        },
    })
}

interface WrapperProps {
    children: React.ReactNode
}

/**
 * All providers wrapper for testing
 */
function AllProviders({ children }: WrapperProps) {
    const queryClient = createTestQueryClient()

    return (
        <QueryClientProvider client={queryClient}>
            {children}
        </QueryClientProvider>
    )
}

/**
 * Custom render that wraps components with all necessary providers
 */
function customRender(
    ui: ReactElement,
    options?: Omit<RenderOptions, 'wrapper'>
) {
    return render(ui, { wrapper: AllProviders, ...options })
}

// Re-export everything from testing-library
export * from '@testing-library/react'
export { customRender as render }

/**
 * Wait for async operations to complete
 */
export async function waitForAsync(ms = 0) {
    await new Promise(resolve => setTimeout(resolve, ms))
}

/**
 * Create a deferred promise for controlling async flow in tests
 */
export function createDeferred<T>() {
    let resolve: (value: T) => void
    let reject: (reason?: unknown) => void

    const promise = new Promise<T>((res, rej) => {
        resolve = res
        reject = rej
    })

    return { promise, resolve: resolve!, reject: reject! }
}
