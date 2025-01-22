import { faBars, faBug } from "@fortawesome/free-solid-svg-icons";
import NavigationIconButton from "./NavigationIconButton";
import { ReactNode } from "react";
import { useLocalization } from "@fluent/react";

type NavigationBarProps = { children?: ReactNode };

/**
 * Bar displayed at the top of screen to allow navigation and supplemental game actions.
 */
export default function NavigationBar({ children }: NavigationBarProps) {
  const { l10n } = useLocalization();
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "row",
        justifyContent: "space-between"
      }}
    >
      <NavigationIconButton icon={faBars} ariaLabel={l10n.getString("menu")} />
      {children}
      <NavigationIconButton
        icon={faBug}
        ariaLabel={l10n.getString("bug-report")}
      />
    </div>
  );
}
