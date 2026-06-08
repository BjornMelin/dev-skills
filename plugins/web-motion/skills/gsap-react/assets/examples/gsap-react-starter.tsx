'use client';

import { useRef } from 'react';
import { gsap } from 'gsap';
import { useGSAP } from '@gsap/react';

gsap.registerPlugin(useGSAP);

/**
 * Renders an intro section and uses useGSAP to animate `.item` elements on mount.
 */
export function Intro() {
  const root = useRef<HTMLElement>(null);
  useGSAP(() => {
    gsap.from('.item', { y: 16, autoAlpha: 0, stagger: 0.05 });
  }, { scope: root });
  return (
    <section ref={root}>
      <p className="item">Motion starts after the component mounts.</p>
    </section>
  );
}
