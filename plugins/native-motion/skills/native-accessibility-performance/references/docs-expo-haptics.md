# Expo Haptics Notes

Source: Expo SDK 56 Haptics docs, checked during the 2026-06-04 audit pass.

Use this file when animation includes tactile feedback or vibration.

## Platform Facts

- Expo SDK 56 bundles `expo-haptics` around 56.0.x.
- `expo-haptics` supports Android, iOS, and web.
- iOS Taptic Engine can do nothing when Low-Power Mode is enabled, the user
  disabled Taptic Engine, the camera is active, or dictation is active.
- Web uses the Web Vibration API and depends on browser support, hardware,
  permission, and foreground/background behavior.
- On Android, Expo docs recommend `performAndroidHapticsAsync` for Android
  haptic feedback because it uses the device haptics engine and does not require
  `VIBRATE` permission.

## Accessibility Rules

- Haptics should confirm an intentional user action or outcome. Do not tie them
  to decorative loops, autoplay, scroll frames, or every animation tick.
- Avoid high-frequency haptic feedback. Use subtle Android types such as
  `Segment_Frequent_Tick` only when the interaction semantics match and test
  real hardware.
- Haptic failure must not hide state. Always pair haptics with visual and
  accessible state.
- Respect reduced motion and screen-reader context. Reduced motion does not
  automatically disable haptics, but tactile output can still become distracting
  when tied to repeated or decorative movement.

## Validation

- Test on real devices when haptics matter; simulators and web may not prove
  tactile behavior.
- Record skipped haptic proof explicitly, especially for iOS Low-Power Mode or
  Android hardware availability.
- On Android, verify whether `performAndroidHapticsAsync` better matches the
  semantic action than `impactAsync`/`notificationAsync`.
