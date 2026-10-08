import { CalculatorIcon, CoinsIcon } from "lucide-react";
import { Button } from "@storyteller/ui-button";
import { ModelPage } from "@storyteller/ui-model-selector";
import { useCostBreakdownModalStore } from "./cost-breakdown-modal-store";

export interface CostCalculatorButtonProps {
  className?: string;
  modelPage?: ModelPage;
}

export function CostCalculatorButton({
  className,
  modelPage,
}: CostCalculatorButtonProps) {
  const { openModal, estimatedCreditsByPage, estimatedUsdCentsByPage } =
    useCostBreakdownModalStore();

  const credits =
    modelPage != null ? estimatedCreditsByPage[modelPage] : undefined;
  // BYOK providers (Venice) report a real USD estimate instead of credits.
  const usdCents =
    modelPage != null ? estimatedUsdCentsByPage[modelPage] : undefined;

  return (
    <Button
      variant="action"
      onClick={openModal}
      className={className}
      title="Cost Calculator"
    >
      <CalculatorIcon  className="text-base-fg" />
      <span>Costs</span>
      {usdCents != null ? (
        <span className="text-xs font-semibold">
          ${(usdCents / 100).toFixed(2)}
        </span>
      ) : credits != null ? (
        <span className="text-xs font-semibold flex items-center gap-1">
          <CoinsIcon  className="text-base-fg" />
          {credits}
        </span>
      ) : null}
    </Button>
  );
}

export default CostCalculatorButton;
