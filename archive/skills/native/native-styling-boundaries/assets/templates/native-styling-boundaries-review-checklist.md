# Native Styling Boundaries Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Inspect NativeWind/react-native-css versions and Babel/Metro setup.
- [ ] Keep classes statically discoverable and token-driven.
- [ ] Choose style/class ownership before adding Reanimated or CSS transitions.
- [ ] Validate iOS/Android/web behavior where NativeWind support differs.
- [ ] Check gotcha: Web Tailwind assumptions do not always map to React Native style props.
- [ ] Check gotcha: Runtime string concatenation can break class extraction and policy.
- [ ] Check gotcha: Animation ownership should not be split between NativeWind classes and Reanimated shared values.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
