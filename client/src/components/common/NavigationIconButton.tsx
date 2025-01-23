import { IconProp } from "@fortawesome/fontawesome-svg-core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { memo } from "react";

type NavigationIconButtonProps = {
  icon: IconProp;
  ariaLabel: string;
  onPress?: () => void;
};

/**
 * A button with an icon that is used in the navigation bar.
 *
 * @param icon - The icon to display.
 * @param ariaLabel - The aria label for the button.
 */
export default memo(function NavigationIconButton({
  icon,
  ariaLabel,
  onPress,
}: NavigationIconButtonProps) {
  return (
    <button
      aria-label={ariaLabel}
      style={{ marginLeft: "16px", marginRight: "16px" }}
      onClick={onPress}
    >
      <FontAwesomeIcon icon={icon} size="lg" />
    </button>
  );
});
