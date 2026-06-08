import { useEffect, useRef, useState } from 'react';
import { AccessibilityInfo, ActivityIndicator, StyleSheet, Text, View } from 'react-native';
import { Fit, RiveView, useRive, useRiveFile } from '@rive-app/react-native';

/**
 * Supported source shapes for native Rive assets.
 */
type RiveSource = string | number | { uri: string } | ArrayBuffer;

/**
 * Props for the native Rive badge example.
 */
type NativeRiveBadgeProps = {
  source: RiveSource;
  stateMachineName?: string;
  onError?: (error: unknown) => void;
};

/**
 * Renders a native Rive badge with loading and error fallbacks.
 *
 * @param source - Rive asset source, such as a local require, URI, or ArrayBuffer.
 * @param stateMachineName - Name of the state machine to control.
 * @param onError - Optional callback invoked for loading or runtime errors.
 * @returns A Rive view once the asset loads, otherwise a native fallback view.
 */
export function NativeRiveBadge({
  source,
  stateMachineName = 'badgeState',
  onError,
}: NativeRiveBadgeProps) {
  const { riveFile, isLoading, error } = useRiveFile(source);
  const { setHybridRef } = useRive();
  const onErrorRef = useRef(onError);
  const [isReduceMotion, setIsReduceMotion] = useState(true);

  useEffect(() => {
    onErrorRef.current = onError;
  }, [onError]);

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

  useEffect(() => {
    if (error) onErrorRef.current?.(error);
  }, [error]);

  if (isLoading) {
    return (
      <View accessibilityRole="progressbar" style={styles.fallback}>
        <ActivityIndicator />
      </View>
    );
  }

  if (error || !riveFile) {
    return (
      <View accessibilityRole="image" style={styles.fallback}>
        <Text style={styles.errorText}>Rive asset unavailable</Text>
      </View>
    );
  }

  if (isReduceMotion) {
    return (
      <View
        accessibilityLabel="Rive badge animation paused for reduced motion"
        accessibilityRole="image"
        style={styles.fallback}
      >
        <Text style={styles.staticText}>Badge</Text>
      </View>
    );
  }

  return (
    <RiveView
      file={riveFile}
      hybridRef={setHybridRef}
      stateMachineName={stateMachineName}
      fit={Fit.Contain}
      onError={(runtimeError) => onErrorRef.current?.(runtimeError)}
      style={styles.rive}
    />
  );
}

const styles = StyleSheet.create({
  rive: {
    width: 120,
    height: 120,
  },
  fallback: {
    width: 120,
    height: 120,
    alignItems: 'center',
    justifyContent: 'center',
  },
  errorText: {
    fontSize: 16,
  },
  staticText: {
    fontSize: 16,
    fontWeight: '600',
  },
});
