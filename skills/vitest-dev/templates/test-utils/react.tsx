import React, { PropsWithChildren } from 'react'
import { render, RenderOptions } from '@testing-library/react'

function Providers({ children }: PropsWithChildren) {
  // Add real providers here:
  // - ThemeProvider
  // - QueryClientProvider
  // - Redux Provider
  return <>{children}</>
}

export function renderWithProviders(
  ui: React.ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>,
) {
  return render(ui, { wrapper: Providers, ...options })
}
