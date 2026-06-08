import { AccessibilityInfo } from 'react-native';

export async function shouldReduceNativeMotion() {
  return AccessibilityInfo.isReduceMotionEnabled();
}
