import React from 'react'
import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

// Mock next/navigation for client components
const push = vi.fn()
vi.mock('next/navigation', () => ({
  useRouter: () => ({ push }),
}))

import { ClientNavButton } from '../app/client-nav-button'

describe('<ClientNavButton />', () => {
  it('navigates on click', async () => {
    const user = userEvent.setup()
    render(<ClientNavButton />)

    await user.click(screen.getByRole('button', { name: /go to dashboard/i }))
    expect(push).toHaveBeenCalledWith('/dashboard')
  })
})
