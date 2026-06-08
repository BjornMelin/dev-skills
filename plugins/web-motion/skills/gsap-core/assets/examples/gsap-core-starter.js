import { gsap } from 'gsap';

const tween = gsap.to('.card', {
  autoAlpha: 1,
  y: 0,
  duration: 0.45,
  ease: 'power2.out',
});

export function cleanup() {
  tween.kill();
}
