export function fadeIn(node: HTMLElement, reduceMotion: boolean) {
  const animation = node.animate(
    [{ opacity: 0, transform: reduceMotion ? 'none' : 'translateY(8px)' }, { opacity: 1, transform: 'none' }],
    { duration: reduceMotion ? 1 : 180, easing: 'ease-out', fill: 'both' },
  );
  return () => animation.cancel();
}
