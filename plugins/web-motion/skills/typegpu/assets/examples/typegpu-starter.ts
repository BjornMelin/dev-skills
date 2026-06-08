import tgpu, { d } from 'typegpu';

const Particle = d.struct({
  position: d.vec2f,
  velocity: d.vec2f,
});

export async function createParticles(count: number) {
  const gpu = (globalThis.navigator as { gpu?: { requestAdapter: () => Promise<unknown> } } | undefined)?.gpu;
  if (!gpu) {
    return { ok: false as const, reason: 'webgpu-unavailable' };
  }

  const adapter = await gpu.requestAdapter();
  if (!adapter) {
    return { ok: false as const, reason: 'webgpu-adapter-unavailable' };
  }

  const root = await tgpu.init();
  const particles = root.createBuffer(d.arrayOf(Particle, count)).$usage('storage');
  return { ok: true as const, root, particles };
}
