import { useEffect, useState } from 'react';
import { AccessibilityInfo } from 'react-native';
import LottieView from 'lottie-react-native';

/**
 * Renders a success animation that autoplays only when reduced motion is off and never loops.
 *
 * @returns A Lottie success animation component.
 */
export function SuccessAnimation() {
  const [isReduceMotion, setIsReduceMotion] = useState(true);

  useEffect(() => {
    let mounted = true;

    AccessibilityInfo.isReduceMotionEnabled()
      .then((enabled) => {
        if (mounted) setIsReduceMotion(enabled);
      })
      .catch(() => {
        if (mounted) setIsReduceMotion(true);
      });

    const subscription = AccessibilityInfo.addEventListener(
      'reduceMotionChanged',
      setIsReduceMotion,
    );

    return () => {
      mounted = false;
      subscription.remove();
    };
  }, []);

  return (
    <LottieView
      source={require('./success.json')}
      autoPlay={!isReduceMotion}
      loop={false}
      accessibilityLabel="Success"
    />
  );
}
