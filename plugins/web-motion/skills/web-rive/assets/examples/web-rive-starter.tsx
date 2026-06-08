'use client';

import { useEffect, useState } from 'react';
import { useRive } from '@rive-app/react-webgl2';

function usePrefersReducedMotion() {
  const [reduceMotion, setReduceMotion] = useState(true);

  useEffect(() => {
    const media = window.matchMedia('(prefers-reduced-motion: reduce)');
    const update = () => setReduceMotion(media.matches);
    update();
    media.addEventListener('change', update);
    return () => media.removeEventListener('change', update);
  }, []);

  return reduceMotion;
}

export function RiveLogo() {
  const reduceMotion = usePrefersReducedMotion();

  if (reduceMotion) {
    return (
      <div style={{ width: 320, maxWidth: '100%', aspectRatio: '1 / 1' }} aria-label="Logo">
        Logo
      </div>
    );
  }

  return <AnimatedRiveLogo />;
}

function AnimatedRiveLogo() {
  const { RiveComponent } = useRive({
    src: '/motion/logo.riv',
    stateMachines: 'hero',
    autoplay: true,
  });

  return (
    <div style={{ width: 320, maxWidth: '100%', aspectRatio: '1 / 1' }}>
      <RiveComponent aria-label="Animated logo" style={{ width: '100%', height: '100%' }} />
    </div>
  );
}
