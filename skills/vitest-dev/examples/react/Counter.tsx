import React from 'react'

export function Counter() {
  const [count, setCount] = React.useState(0)
  return (
    <div>
      <p aria-label="count">Count: {count}</p>
      <button type="button" onClick={() => setCount((c) => c + 1)}>
        Increment
      </button>
    </div>
  )
}
