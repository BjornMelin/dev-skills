import { gsap } from 'gsap';

const progressToRotation = gsap.utils.pipe(
  gsap.utils.clamp(0, 1),
  gsap.utils.mapRange(0, 1, -12, 12),
  gsap.utils.snap(0.5),
);

export const rotationForProgress = (progress) => progressToRotation(progress);
