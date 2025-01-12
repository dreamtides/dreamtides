import { faBars, faBug } from "@fortawesome/free-solid-svg-icons";
import NavigationIconButton from "./NavigationIconButton";

/**
 * Bar displayed at the top of screen to allow navigation and supplemental game actions.
 */
export default function NavigationBar() {
  return (
    <div className="flex flex-row justify-between">
      <NavigationIconButton icon={faBars} ariaLabel="Menu" />
      <NavigationIconButton icon={faBug} ariaLabel="Bug Report" />
    </div>
  );
}
