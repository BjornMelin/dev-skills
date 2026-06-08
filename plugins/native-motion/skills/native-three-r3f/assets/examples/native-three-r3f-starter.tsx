import { Canvas } from '@react-three/fiber/native';
import { View } from 'react-native';

export function NativeScene() {
  return (
    <View style={{ width: 240, height: 240 }}>
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
