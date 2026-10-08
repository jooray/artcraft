import { ArrowUpIcon, CoinsIcon, LoaderCircleIcon } from "lucide-react";
import { DynamicIcon } from "@storyteller/icons";
import { ButtonHTMLAttributes } from "react";
import { twMerge } from "tailwind-merge";
import { Tooltip } from "@storyteller/ui-tooltip";

interface GenerateIconButtonProps extends Omit<
  ButtonHTMLAttributes<HTMLButtonElement>,
  "children"
> {
  credits?: number | null;
  /** BYOK providers report a real USD estimate (in cents) instead of credits. */
  costUsdCents?: number | null;
  loading?: boolean;
}

/**
 * Compact generate control: credit cost as plain text next to a circular
 * arrow-up button. Desktop promptboxes use this; the mobile form keeps the
 * labeled `GenerateButton` pill.
 */
export const GenerateIconButton = ({
  credits,
  costUsdCents,
  loading,
  className,
  disabled,
  ...rest
}: GenerateIconButtonProps) => {
  const isDisabled = disabled || loading;

  const hasUsdEstimate = costUsdCents != null;
  const label = hasUsdEstimate
    ? `$${(costUsdCents / 100).toFixed(2)}`
    : credits;
  const tooltip = hasUsdEstimate
    ? `Estimated ${label} on your provider account`
    : credits != null
      ? `${credits} credit${credits !== 1 ? "s" : ""} cost`
      : null;

  return (
    <div className={twMerge("flex shrink-0 items-center gap-2.5", className)}>
      {label != null && tooltip && (
        <Tooltip
          content={tooltip}
          position="top"
          className="z-50"
        >
          <span
            className={twMerge(
              "flex items-center gap-1.5 ms-1.5 text-[13px] font-semibold tabular-nums text-base-fg/80 transition-opacity",
              isDisabled && "opacity-50",
            )}
          >
            {!hasUsdEstimate && <CoinsIcon className="text-xs" />}
            {label}
          </span>
        </Tooltip>
      )}

      <button
        type="button"
        className="flex h-9 w-9 items-center justify-center rounded-full bg-primary text-white shadow-sm transition-all duration-150 hover:bg-primary-400 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40 disabled:active:scale-100"
        disabled={isDisabled}
        {...rest}
      >
        <DynamicIcon
          icon={loading ? LoaderCircleIcon : ArrowUpIcon}
          className={loading ? "animate-spin" : undefined}
        />
      </button>
    </div>
  );
};

export default GenerateIconButton;
