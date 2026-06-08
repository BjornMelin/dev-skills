'use client';

import { Canvas } from '@react-three/fiber';
import { Suspense } from 'react';

export function SceneSurface() {
  return (
    <Canvas dpr={[1, 1.5]} frameloop="demand" camera={{ position: [0, 0, 6], fov: 45 }} fallback={<img src="/scene-fallback.jpg" alt="" />}>
      <Suspense fallback={null}>{/* scene */}</Suspense>
    </Canvas>
  );
}
