import { IconProp } from "@fortawesome/fontawesome-svg-core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Button } from "@nextui-org/react";
import type { PressEvent } from "@react-types/shared";
import { memo } from "react";

type NavigationIconButtonProps = {
  icon: IconProp;
  ariaLabel: string;
  onPress?: (e: PressEvent) => void;
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
    <Button
      isIconOnly
      aria-label={ariaLabel}
      variant="light"
      size="lg"
      className="m-1"
      radius="full"
      onPress={onPress}
    >
      <FontAwesomeIcon icon={icon} size="lg" />
    </Button>
  );
});
