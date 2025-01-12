import { faBars, faBug } from "@fortawesome/free-solid-svg-icons";
import NavigationIconButton from "./NavigationIconButton";
import { ReactNode } from "react";

type NavigationBarProps = { children?: ReactNode };

/**
 * Bar displayed at the top of screen to allow navigation and supplemental game actions.
 */
export default function NavigationBar({ children }: NavigationBarProps) {
  return (
    <div className="flex flex-row justify-between">
      <NavigationIconButton icon={faBars} ariaLabel="Menu" />
      {children}
      <NavigationIconButton icon={faBug} ariaLabel="Bug Report" />
    </div>
  );
}
