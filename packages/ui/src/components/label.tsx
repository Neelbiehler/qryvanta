import * as React from "react";

import { cn } from "../lib/cn";

type LabelProps = Omit<React.ComponentProps<"label">, "htmlFor"> & {
  htmlFor: string;
};

const Label = React.forwardRef<HTMLLabelElement, LabelProps>(
  ({ className, htmlFor, ...props }, ref) => (
    <label
      ref={ref}
      htmlFor={htmlFor}
      className={cn("text-sm font-medium leading-none", className)}
      {...props}
    />
  ),
);

Label.displayName = "Label";

export { Label };
