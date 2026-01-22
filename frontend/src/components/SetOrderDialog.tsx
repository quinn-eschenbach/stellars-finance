import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Loader2 } from "lucide-react";
import { toast } from "sonner";
import {
  useCreateStopLoss,
  useCreateTakeProfit,
  useMinExecutionFee,
  FormattedPosition,
} from "@/hooks/usePositionManager";
import { usePrices } from "@/contexts/PriceContext";

interface SetOrderDialogProps {
  position: FormattedPosition;
  orderType: "stopLoss" | "takeProfit";
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

const SetOrderDialog = ({
  position,
  orderType,
  open,
  onOpenChange,
}: SetOrderDialogProps) => {
  const { getPrice } = usePrices();
  const { data: minFee } = useMinExecutionFee();
  const createStopLossMutation = useCreateStopLoss();
  const createTakeProfitMutation = useCreateTakeProfit();

  const [triggerPrice, setTriggerPrice] = useState("");
  const [closePercentage, setClosePercentage] = useState([100]);

  // Get current price for the market (using XLM for MVP)
  const priceData = getPrice("XLMUSDT");
  const currentPrice = priceData?.price || position.entryPrice;

  const isStopLoss = orderType === "stopLoss";
  const title = isStopLoss ? "Set Stop Loss" : "Set Take Profit";
  const description = isStopLoss
    ? "Set a price to automatically close your position to limit losses"
    : "Set a price to automatically close your position to secure profits";

  const validatePrice = (price: number): string | null => {
    if (price <= 0) {
      return "Price must be greater than 0";
    }

    if (isStopLoss) {
      // Stop loss: for longs, trigger below current; for shorts, trigger above current
      if (position.isLong && price >= currentPrice) {
        return "Stop loss must be below current price for long positions";
      }
      if (!position.isLong && price <= currentPrice) {
        return "Stop loss must be above current price for short positions";
      }
    } else {
      // Take profit: for longs, trigger above current; for shorts, trigger below current
      if (position.isLong && price <= currentPrice) {
        return "Take profit must be above current price for long positions";
      }
      if (!position.isLong && price >= currentPrice) {
        return "Take profit must be below current price for short positions";
      }
    }

    return null;
  };

  const handleSubmit = () => {
    const price = parseFloat(triggerPrice);

    const validationError = validatePrice(price);
    if (validationError) {
      toast.error(validationError);
      return;
    }

    const executionFee = minFee?.formatted || 0.1;

    const params = {
      positionId: position.id,
      triggerPrice: price,
      closePercentage: closePercentage[0],
      executionFee,
    };

    const mutation = isStopLoss ? createStopLossMutation : createTakeProfitMutation;

    mutation.mutate(params, {
      onSuccess: () => {
        setTriggerPrice("");
        setClosePercentage([100]);
        onOpenChange(false);
      },
    });
  };

  const isPending = createStopLossMutation.isPending || createTakeProfitMutation.isPending;

  // Calculate suggested prices
  const suggestedPrice = isStopLoss
    ? position.isLong
      ? currentPrice * 0.95 // 5% below for long SL
      : currentPrice * 1.05 // 5% above for short SL
    : position.isLong
    ? currentPrice * 1.1 // 10% above for long TP
    : currentPrice * 0.9; // 10% below for short TP

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>

        <div className="space-y-6 pt-4">
          <div className="p-3 bg-secondary rounded-lg space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Position</span>
              <span className="font-medium">
                {position.isLong ? "Long" : "Short"} {position.leverage.toFixed(1)}x
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Entry Price</span>
              <span className="font-medium">
                ${position.entryPrice.toLocaleString(undefined, {
                  minimumFractionDigits: 2,
                  maximumFractionDigits: 2,
                })}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Current Price</span>
              <span className="font-medium">
                ${currentPrice.toLocaleString(undefined, {
                  minimumFractionDigits: 2,
                  maximumFractionDigits: 2,
                })}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Position Size</span>
              <span className="font-medium">
                ${position.size.toLocaleString(undefined, {
                  minimumFractionDigits: 2,
                  maximumFractionDigits: 2,
                })}
              </span>
            </div>
          </div>

          <div>
            <Label htmlFor="triggerPrice">Trigger Price</Label>
            <div className="flex gap-2 mt-2">
              <Input
                id="triggerPrice"
                type="number"
                placeholder="0.00"
                value={triggerPrice}
                onChange={(e) => setTriggerPrice(e.target.value)}
                disabled={isPending}
              />
              <Button
                variant="outline"
                size="sm"
                onClick={() => setTriggerPrice(suggestedPrice.toFixed(4))}
                disabled={isPending}
              >
                Suggested
              </Button>
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              {isStopLoss
                ? position.isLong
                  ? "Must be below current price"
                  : "Must be above current price"
                : position.isLong
                ? "Must be above current price"
                : "Must be below current price"}
            </p>
          </div>

          <div>
            <div className="flex justify-between mb-2">
              <Label>Close Percentage</Label>
              <span className="text-sm font-medium">{closePercentage[0]}%</span>
            </div>
            <Slider
              value={closePercentage}
              onValueChange={setClosePercentage}
              min={1}
              max={100}
              step={1}
              disabled={isPending}
            />
            <div className="flex justify-between text-xs text-muted-foreground mt-1">
              <span>1%</span>
              <span>100% (Full close)</span>
            </div>
          </div>

          {minFee && (
            <div className="flex justify-between text-sm p-3 bg-secondary rounded-lg">
              <span className="text-muted-foreground">Execution Fee</span>
              <span className="font-medium">${minFee.formatted.toFixed(2)}</span>
            </div>
          )}

          <Button
            onClick={handleSubmit}
            disabled={!triggerPrice || isPending}
            className={`w-full ${
              isStopLoss
                ? "bg-orange-500 hover:bg-orange-600"
                : "bg-green-500 hover:bg-green-600"
            }`}
          >
            {isPending ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin mr-2" />
                Creating...
              </>
            ) : (
              `Create ${isStopLoss ? "Stop Loss" : "Take Profit"}`
            )}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default SetOrderDialog;
