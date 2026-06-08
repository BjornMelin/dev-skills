import { Pressable, Text } from 'react-native';

export function NativeButton() {
  return <Pressable className="rounded-md bg-black px-4 py-3 active:opacity-80"><Text className="text-white">Save</Text></Pressable>;
}
