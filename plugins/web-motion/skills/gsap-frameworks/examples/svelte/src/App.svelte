<script>
  import { onMount, tick } from "svelte";
  import { gsap } from "gsap";

  let container;

  onMount(() => {
    let ctx;
    let cancelled = false;

    (async () => {
      const { SplitText } = await import("gsap/SplitText");
      if (cancelled || !container) return;

      gsap.registerPlugin(SplitText);
      await tick();
      if (cancelled || !container) return;

      ctx = gsap.context(() => {
        const split = SplitText.create(".headline", { type: "chars" });
        gsap.from(split.chars, {
          autoAlpha: 0,
          y: 24,
          stagger: 0.03,
          duration: 0.45,
          ease: "power2.out",
        });

        gsap.from(".item", {
          autoAlpha: 0,
          y: 16,
          stagger: 0.08,
          duration: 0.4,
        });

        return () => split.revert();
      }, container);
    })();

    return () => {
      cancelled = true;
      ctx?.revert();
    };
  });
</script>

<main bind:this={container}>
  <h1 class="headline">GSAP Svelte</h1>
  <p class="item">Scoped selectors stay inside this component.</p>
  <p class="item">Cleanup runs from the synchronous onMount return.</p>
</main>

<style>
  main {
    min-height: 100vh;
    display: grid;
    place-content: center;
    gap: 1rem;
    padding: 2rem;
    font-family: system-ui, sans-serif;
  }

  .headline {
    margin: 0;
    font-size: clamp(2.5rem, 8vw, 6rem);
    line-height: 0.95;
  }

  .item {
    max-width: 34rem;
    margin: 0;
    color: #334155;
  }
</style>
