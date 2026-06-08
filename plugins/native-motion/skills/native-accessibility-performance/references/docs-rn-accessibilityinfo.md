# React Native AccessibilityInfo Notes

Source: React Native AccessibilityInfo docs, checked during the 2026-06-04
audit pass.

Use this file when code needs to react to accessibility settings at runtime.

## Relevant APIs

- `AccessibilityInfo.isReduceMotionEnabled()` returns a promise for the current
  reduce-motion setting.
- `AccessibilityInfo.addEventListener('reduceMotionChanged', handler)` fires
  when the reduce-motion setting changes. On Android, it also reports true
  when Developer Options transition animations are off.
- `AccessibilityInfo.isScreenReaderEnabled()` returns whether a screen reader is
  active.
- `AccessibilityInfo.addEventListener('screenReaderChanged', handler)` reports
  screen-reader setting changes.
- `AccessibilityInfo.announceForAccessibility(message)` posts a screen-reader
  announcement.
- `AccessibilityInfo.announceForAccessibilityWithOptions(message, { queue })`
  can queue iOS announcements.
- `AccessibilityInfo.prefersCrossFadeTransitions()` reports the iOS preference
  for cross-fade transitions when reduce motion is enabled.
- `AccessibilityInfo.sendAccessibilityEvent(ref, eventType)` is the current
  imperative focus/event API. React Native 0.85 marks `setAccessibilityFocus`
  deprecated in favor of `sendAccessibilityEvent(..., 'focus')`.

## Audit Implications

- Use `AccessibilityInfo` when the app must update without restart after the
  user toggles reduced motion or a screen reader.
- Always remove subscriptions in cleanup.
- Do not spam `announceForAccessibility` from animation frames, loops, scroll
  handlers, or rapidly changing counters.
- If an animation changes focus or mounts new content, verify behavior with
  VoiceOver/TalkBack rather than relying on static code review.
- If iOS cross-fade preference matters, check `prefersCrossFadeTransitions()`
  before forcing a custom spatial transition.

## Minimal Pattern

```tsx
useEffect(() => {
  let mounted = true;

  AccessibilityInfo.isReduceMotionEnabled().then((enabled) => {
    if (mounted) setReduceMotion(enabled);
  });

  const subscription = AccessibilityInfo.addEventListener(
    'reduceMotionChanged',
    setReduceMotion,
  );

  return () => {
    mounted = false;
    subscription.remove();
  };
}, []);
```
