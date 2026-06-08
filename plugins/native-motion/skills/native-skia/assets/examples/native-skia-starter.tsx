import { Canvas, Circle } from '@shopify/react-native-skia';

export function Dot() {
  return <Canvas style={{ width: 96, height: 96 }}><Circle cx={48} cy={48} r={24} color="black" /></Canvas>;
}
