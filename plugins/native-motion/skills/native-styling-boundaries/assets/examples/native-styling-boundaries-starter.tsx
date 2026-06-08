import { Pressable, Text } from 'react-native';

/**
 * Renders a NativeWind button example with static styling classes and a press handler.
 *
 * @param onPress - Optional callback invoked when the button is pressed.
 * @returns A styled Pressable fixture for class safety checks.
 */
export function NativeButton({ onPress = () => undefined }: { onPress?: () => void }) {
  return (
    <Pressable
      accessibilityRole="button"
      className="min-h-[44px] justify-center rounded-md bg-black px-4 py-3 active:opacity-80"
      onPress={onPress}
    >
      <Text className="text-white">Save</Text>
    </Pressable>
  );
}
