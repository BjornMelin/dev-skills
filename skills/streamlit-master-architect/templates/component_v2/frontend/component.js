export default function smaCounter(component) {
  const { data, parentElement, setStateValue, setTriggerValue } = component

  // Basic resilient access: Python kwargs are passed as an object.
  const label = (data && data.label) ? String(data.label) : "Click"

  // Avoid clobbering parentElement; render into a container.
  const root = document.createElement("div")
  root.style.display = "flex"
  root.style.gap = "0.5rem"
  root.style.alignItems = "center"
  root.style.fontFamily = "system-ui, -apple-system, Segoe UI, Roboto, sans-serif"

  const button = document.createElement("button")
  button.textContent = label
  button.type = "button"

  const countEl = document.createElement("span")
  countEl.textContent = "0"

  // Mount
  root.appendChild(button)
  root.appendChild(countEl)
  parentElement.appendChild(root)

  let count = 0
  const onClick = () => {
    count += 1
    countEl.textContent = String(count)
    setStateValue("count", count)
    setTriggerValue("clicked", true)
  }
  button.addEventListener("click", onClick)

  // Cleanup on unmount
  return () => {
    button.removeEventListener("click", onClick)
    root.remove()
  }
}

