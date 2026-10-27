import { describe, expect, it } from 'vitest'
import { add } from './add'

describe('add', () => {
  it('adds positive numbers', () => {
    expect(add(1, 2)).toBe(3)
  })

  it('adds negative numbers', () => {
    expect(add(-1, -2)).toBe(-3)
  })

  it('handles zeros', () => {
    expect(add(0, 5)).toBe(5)
    expect(add(5, 0)).toBe(5)
  })
})
