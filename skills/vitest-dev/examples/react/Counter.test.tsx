import React from 'react'
import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Counter } from './Counter'

describe('<Counter />', () => {
  it('increments when the user clicks', async () => {
    const user = userEvent.setup()
    render(<Counter />)

    expect(screen.getByLabelText('count')).toHaveTextContent('Count: 0')
    await user.click(screen.getByRole('button', { name: /increment/i }))
    expect(screen.getByLabelText('count')).toHaveTextContent('Count: 1')
  })
})
