"use client";

import { memo, type ComponentProps } from "react";
import { Streamdown } from "streamdown";

type Props = ComponentProps<typeof Streamdown>;

function Response({ className, ...props }: Props) {
  const base = "prose prose-sm dark:prose-invert max-w-none";
  return (
    <Streamdown
      className={className ? `${base} ${className}` : base}
      {...props}
    />
  );
}

export default memo(Response);

