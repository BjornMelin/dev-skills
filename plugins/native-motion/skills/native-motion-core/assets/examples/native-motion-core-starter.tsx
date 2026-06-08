import { useEffect } from 'react';
import Animated, {
  ReduceMotion,
  useAnimatedStyle,
  useSharedValue,
  withTiming,
} from 'react-native-reanimated';

export function NativeMotionCoreStarter() {
  const progress = useSharedValue(0);
  const style = useAnimatedStyle(() => ({
    opacity: progress.value,
    transform: [{ scale: 0.96 + progress.value * 0.04 }],
  }));

  useEffect(() => {
    progress.value = withTiming(1, {
      duration: 180,
      reduceMotion: ReduceMotion.System,
    });
  }, [progress]);

  return (
    <Animated.View
      accessibilityRole="image"
      style={[
        {
          width: 64,
          height: 64,
          borderRadius: 16,
          backgroundColor: '#2563eb',
        },
        style,
      ]}
    />
  );
}
