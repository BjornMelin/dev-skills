import { useEffect } from 'react';
import { ActivityIndicator, StyleSheet, Text, View } from 'react-native';
import { Fit, RiveView, useRive, useRiveFile } from '@rive-app/react-native';

type RiveSource = string | number | { uri: string } | ArrayBuffer;

type NativeRiveBadgeProps = {
  source: RiveSource;
  stateMachineName?: string;
  onError?: (error: unknown) => void;
};

export function NativeRiveBadge({
  source,
  stateMachineName = 'badgeState',
  onError,
}: NativeRiveBadgeProps) {
  const { riveFile, isLoading, error } = useRiveFile(source);
  const { setHybridRef } = useRive();

  useEffect(() => {
    if (error) onError?.(error);
  }, [error, onError]);

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

  return riveFile ? (
    <RiveView
      file={riveFile}
      hybridRef={setHybridRef}
      stateMachineName={stateMachineName}
      fit={Fit.Contain}
      onError={(runtimeError) => onError?.(runtimeError)}
      style={styles.rive}
    />
  ) : null;
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
    fontSize: 12,
  },
});
