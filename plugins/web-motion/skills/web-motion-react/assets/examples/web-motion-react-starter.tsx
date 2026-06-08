'use client';

import { AnimatePresence, motion, useReducedMotion } from 'motion/react';

export function Notice({ open }: { open: boolean }) {
  const reduce = useReducedMotion();
  return (
    <AnimatePresence>
      {open ? <motion.div initial={{ opacity: 0, y: reduce ? 0 : 8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} /> : null}
    </AnimatePresence>
  );
}
