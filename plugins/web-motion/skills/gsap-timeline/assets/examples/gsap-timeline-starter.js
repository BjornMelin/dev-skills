import { gsap } from 'gsap';

const tl = gsap.timeline({ defaults: { duration: 0.4, ease: 'power2.out' } });
tl.addLabel('intro')
  .from('.title', { y: 16, autoAlpha: 0 }, 'intro')
  .from('.body', { y: 12, autoAlpha: 0 }, '<0.08')
  .to('.cta', { scale: 1 }, '+=0.1');
