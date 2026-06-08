import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

const pluginMap = {
  CustomEase: () => import("gsap/CustomEase"),
  Draggable: () => import("gsap/Draggable"),
  CSSRulePlugin: () => import("gsap/CSSRulePlugin"),
  EaselPlugin: () => import("gsap/EaselPlugin"),
  EasePack: () => import("gsap/EasePack"),
  Flip: () => import("gsap/Flip"),
  MotionPathPlugin: () => import("gsap/MotionPathPlugin"),
  Observer: () => import("gsap/Observer"),
  PixiPlugin: () => import("gsap/PixiPlugin"),
  ScrollToPlugin: () => import("gsap/ScrollToPlugin"),
  ScrollTrigger: () => import("gsap/ScrollTrigger"),
  TextPlugin: () => import("gsap/TextPlugin"),
  DrawSVGPlugin: () => import("gsap/DrawSVGPlugin"),
  Physics2DPlugin: () => import("gsap/Physics2DPlugin"),
  PhysicsPropsPlugin: () => import("gsap/PhysicsPropsPlugin"),
  ScrambleTextPlugin: () => import("gsap/ScrambleTextPlugin"),
  CustomBounce: () => import("gsap/CustomBounce"),
  CustomWiggle: () => import("gsap/CustomWiggle"),
  GSDevTools: () => import("gsap/GSDevTools"),
  InertiaPlugin: () => import("gsap/InertiaPlugin"),
  MorphSVGPlugin: () => import("gsap/MorphSVGPlugin"),
  MotionPathHelper: () => import("gsap/MotionPathHelper"),
  ScrollSmoother: () => import("gsap/ScrollSmoother"),
  SplitText: () => import("gsap/SplitText"),
} as const;

type PluginMap = typeof pluginMap;
type Plugins = keyof PluginMap;
type PluginModule<K extends Plugins> = Awaited<ReturnType<PluginMap[K]>>;
type PluginExport<K extends Plugins> = PluginModule<K>[K & keyof PluginModule<K>];

/**
 * Provides GSAP with ScrollTrigger registered and a typed lazy plugin loader.
 */
export function useGSAP() {
  gsap.registerPlugin(ScrollTrigger);

  async function lazyLoadPlugin<K extends Plugins>(plugin: K): Promise<PluginExport<K>> {
    const loader = pluginMap[plugin];
    const module = await loader();
    const loadedPlugin = (module as Record<K, PluginExport<K>>)[plugin];
    gsap.registerPlugin(loadedPlugin);
    return loadedPlugin;
  }

  return { gsap, ScrollTrigger, lazyLoadPlugin };
}
