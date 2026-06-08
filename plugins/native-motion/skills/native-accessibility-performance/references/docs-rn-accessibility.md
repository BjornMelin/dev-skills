# React Native Accessibility Notes

Source: React Native Accessibility docs, checked during the 2026-06-04 audit
pass.

Use this file when animation changes semantic content, focus, dynamic text, or
screen-reader announcements.

## Properties That Matter For Motion

- `accessible` groups children into one accessibility element. Animated
  grouping changes can alter screen-reader navigation.
- `accessibilityLabel`, `accessibilityHint`, `accessibilityRole`, and
  `accessibilityState` must remain true while visual state animates.
- `accessibilityLiveRegion` is Android-only and supports `none`, `polite`, and
  `assertive`. Use `polite` for normal dynamic updates; reserve `assertive` for
  urgent interruptions.
- `importantForAccessibility` can hide decorative or duplicate animated
  content from TalkBack.
- Accessibility actions should continue to work when animation changes
  collapsed/expanded, reorder, or gesture state.
- `accessibilityIgnoresInvertColors` is iOS-only and should be used sparingly,
  usually for photos or media that should not invert. Do not use it to hide
  motion contrast problems.
- `experimental_accessibilityOrder` can change reading order. Animated visual
  reorder and explicit accessibility order need screen-reader proof together.

## Motion Review Rules

- Do not animate semantic order. If a visual reorder is needed, make sure the
  accessibility tree order still matches the user's expected navigation order.
- Avoid animated text churn inside live regions. Prefer a single settled
  announcement after the state change.
- Hide decorative animated layers from the accessibility tree when a stable
  accessible control or label already communicates the same state.
- Verify dynamic type with animated containers that clip, expand, collapse, or
  transform text.
- Keep focus targets stable under transform and feature-flag fast paths; if a
  touch target moves visually, test real touch and screen-reader activation.
- Avoid changing `accessible`, `importantForAccessibility`, role, or state as an
  animation side effect unless that semantic state truly changed.

## Manual Checks

- VoiceOver and TalkBack can enter, operate, and leave the animated surface.
- Reduced motion does not remove essential information.
- Rapid state changes do not queue misleading announcements.
- Dynamic type does not clip or overlap text during and after motion.
