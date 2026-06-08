import { Canvas } from '@react-three/fiber/native';
import { View } from 'react-native';

/**
 * Renders a small accessible React Three Fiber native scene for GPU smoke tests.
 *
 * @returns A view containing a native R3F canvas scene.
 */
export function NativeScene() {
  return (
    <View
      accessibilityLabel="Native 3D scene with a blue cube"
      accessibilityRole="image"
      style={{ width: 240, height: 240 }}
    >
      <Canvas>
        <ambientLight intensity={0.8} />
        <mesh>
          <boxGeometry args={[1, 1, 1]} />
          <meshStandardMaterial color="#38bdf8" />
        </mesh>
      </Canvas>
    </View>
  );
}
