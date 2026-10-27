'use client'

import React from 'react'
import { useRouter } from 'next/navigation'

export function ClientNavButton() {
  const router = useRouter()
  return (
    <button type="button" onClick={() => router.push('/dashboard')}>
      Go to dashboard
    </button>
  )
}
