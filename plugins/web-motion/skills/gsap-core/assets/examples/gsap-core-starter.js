import { gsap } from 'gsap';

const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
const tween = reduceMotion
  ? gsap.set('.card', { autoAlpha: 1, y: 0 })
  : gsap.to('.card', {
      autoAlpha: 1,
      y: 0,
      duration: 0.45,
      ease: 'power2.out',
    });

export function cleanup() {
  tween.kill();
}
