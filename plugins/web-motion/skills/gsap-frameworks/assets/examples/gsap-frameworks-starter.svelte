<script>
  import { onMount } from 'svelte';
  import { gsap } from 'gsap';

  let root;

  onMount(() => {
    const ctx = gsap.context(() => {
      gsap.from('.item', { y: 16, autoAlpha: 0, stagger: 0.06 });
    }, root);
    return () => ctx.revert();
  });
</script>

<section bind:this={root}>
  <slot />
</section>
