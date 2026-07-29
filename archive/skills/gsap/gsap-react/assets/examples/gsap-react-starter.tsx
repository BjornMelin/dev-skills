'use client';

import { useRef } from 'react';
import { gsap } from 'gsap';
import { useGSAP } from '@gsap/react';

gsap.registerPlugin(useGSAP);

function prefersReducedMotion() {
  return globalThis.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false;
}

/**
 * Renders an intro section and uses useGSAP to animate `.item` elements on mount.
 *
 * @returns The intro section with a scoped animated item.
 */
export function Intro() {
  const root = useRef<HTMLElement>(null);
  useGSAP(() => {
    if (prefersReducedMotion()) {
      gsap.set('.item', { autoAlpha: 1, y: 0 });
      return;
    }
    gsap.from('.item', { y: 16, autoAlpha: 0, stagger: 0.05 });
  }, { scope: root });
  return (
    <section ref={root}>
      <p className="item">Motion starts after the component mounts.</p>
    </section>
  );
}
