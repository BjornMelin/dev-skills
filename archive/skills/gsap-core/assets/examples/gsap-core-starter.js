import { gsap } from 'gsap';

const mm = gsap.matchMedia();
const tweens = [];

mm.add('(prefers-reduced-motion: reduce)', () => {
  gsap.set('.card', { autoAlpha: 1, y: 0 });
});

mm.add('(prefers-reduced-motion: no-preference)', () => {
  const tween = gsap.to('.card', {
    autoAlpha: 1,
    y: 0,
    duration: 0.45,
    ease: 'power2.out',
  });
  tweens.push(tween);
});

export function cleanup() {
  tweens.forEach((tween) => tween.kill());
  mm.revert();
}
