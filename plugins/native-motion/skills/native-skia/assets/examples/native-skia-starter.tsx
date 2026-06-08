import { Canvas, Circle } from '@shopify/react-native-skia';

const canvasStyle = { width: 96, height: 96 };

/**
 * Renders a minimal Skia canvas with a static dot for native rendering smoke tests.
 *
 * @returns A Skia canvas component.
 */
export function Dot() {
  return (
    <Canvas style={canvasStyle}>
      <Circle cx={48} cy={48} r={24} color="black" />
    </Canvas>
  );
}
