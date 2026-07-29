<script>
  import { onMount } from 'svelte';
  import { gsap } from 'gsap';

  let root;

  onMount(() => {
    const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    const ctx = gsap.context(() => {
      if (!reduceMotion) {
        gsap.from('.item', { y: 16, autoAlpha: 0, stagger: 0.06 });
      }
    }, root);
    return () => ctx.revert();
  });
</script>

<section bind:this={root}>
  <slot />
</section>
