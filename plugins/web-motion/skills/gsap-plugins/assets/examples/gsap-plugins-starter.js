import { gsap } from 'gsap';
import { Flip } from 'gsap/Flip';

gsap.registerPlugin(Flip);

export function flipToNextState(target, mutateDom) {
  const state = Flip.getState(target);
  mutateDom();
  return Flip.from(state, { duration: 0.35, ease: 'power2.out' });
}
