'use client';

import { useEffect, useRef, useState } from 'react';
import lottie from 'lottie-web';

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

/**
 * Renders a Lottie animation badge that respects reduced-motion preferences.
 *
 * @param path - URL or path to the Lottie JSON asset.
 */
export function LottieBadge({ path }: { path: string }) {
  const host = useRef<HTMLDivElement>(null);
  const reduceMotion = usePrefersReducedMotion();

  useEffect(() => {
    if (!host.current || reduceMotion) return;
    const animation = lottie.loadAnimation({ container: host.current, renderer: 'svg', loop: false, autoplay: true, path });
    return () => animation.destroy();
  }, [path, reduceMotion]);

  return <div ref={host} aria-label="Status animation">{reduceMotion ? 'Status' : null}</div>;
}
