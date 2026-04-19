import type { ButtonHTMLAttributes } from "react";
import type { ButtonVariant } from "../design-tokens";
import { buttonVariant } from "../design-tokens";

/**
 * Shared button primitive that funnels every battle-screen button through the
 * four-variant taxonomy in `design-tokens.ts` (FIND-10-5). Stage-3 agents
 * should prefer this component; fall back to `buttonVariant("primary")` as a
 * className only when integrating with legacy button sites that cannot yet
 * migrate.
 */
export function Button({
  variant,
  className,
  type = "button",
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant: ButtonVariant }) {
  const classes = className
    ? `${buttonVariant(variant)} ${className}`
    : buttonVariant(variant);
  return <button type={type} className={classes} {...rest} />;
}
