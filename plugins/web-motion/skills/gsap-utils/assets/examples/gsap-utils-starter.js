import { gsap } from 'gsap';

const progressToRotation = gsap.utils.pipe(
  gsap.utils.clamp(0, 1),
  gsap.utils.mapRange(0, 1, -12, 12),
  gsap.utils.snap(0.5),
);

/**
 * Delegates to progressToRotation(progress) to convert normalized progress into rotation.
 *
 * @param {number} progress - Normalized progress value from 0 to 1.
 * @returns {number} Rotation in degrees, clamped, mapped to [-12, 12], and snapped to 0.5 increments.
 */
export const rotationForProgress = (progress) => progressToRotation(progress);
