import { AccessibilityInfo } from 'react-native';

/**
 * Checks whether the OS-level reduce motion accessibility setting is enabled.
 *
 * @returns A promise that resolves to true when reduce motion is enabled.
 */
export async function shouldReduceNativeMotion() {
  return AccessibilityInfo.isReduceMotionEnabled();
}
