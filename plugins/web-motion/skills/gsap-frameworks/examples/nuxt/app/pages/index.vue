<script setup>
import { onMounted, onUnmounted, ref } from "vue";

const { gsap, lazyLoadPlugin } = useGSAP();

const container = ref(null);
let ctx;
let cancelled = false;

onMounted(async () => {
  if (!container.value) return;
  const SplitText = await lazyLoadPlugin("SplitText");
  if (cancelled || !container.value) return;

  ctx = gsap.context(() => {
    const split = SplitText.create(".headline", {
      type: "chars",
    });

    gsap.from(split.chars, {
      autoAlpha: 0,
      y: -50,
      xPercent: -100,
      rotation: -45,
      ease: "power1.inOut",
      stagger: {
        amount: 0.3,
      },
    });

    gsap.set(".headline", { autoAlpha: 1 });

    return () => split.revert();
  }, container.value);
});

onUnmounted(() => {
  cancelled = true;
  ctx?.revert();
});
</script>

<template>
  <main ref="container">
    <h1 class="headline">GSAP SplitText</h1>
  </main>
</template>

<style scoped>
  main {
    width: 100%;
    height: 100vh;
    display: flex;
    justify-content: center;
    align-items: center;
    font-family: 'Trebuchet MS', 'Lucida Sans Unicode', 'Lucida Grande', 'Lucida Sans', Arial, sans-serif;
    font-size: 18px;
  }

  main h1 {
    opacity: 0;
    visibility: hidden;
  }
</style>
