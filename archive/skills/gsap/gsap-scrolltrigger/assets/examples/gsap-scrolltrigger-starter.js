import { gsap } from 'gsap';
import { ScrollTrigger } from 'gsap/ScrollTrigger';

gsap.registerPlugin(ScrollTrigger);

const timeline = gsap.timeline({
  scrollTrigger: {
    trigger: '.panel',
    start: 'top center',
    end: 'bottom center',
    scrub: true,
  },
});

timeline.to('.panel', { x: 120, ease: 'none' });
