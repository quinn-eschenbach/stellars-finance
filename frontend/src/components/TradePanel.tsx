import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { toast } from "sonner";
import {
  useOpenPosition,
  useCreateLimitOrder,
  useUserTokenBalance,
  useMinExecutionFee,
} from "@/hooks/usePositionManager";
import { useWallet } from "@/hooks/useWallet";
import { Loader2 } from "lucide-react";

interface TradePanelProps {
  asset: string;
  currentPrice: number;
  marketId: number;
}

type OrderMode = "market" | "limit";

const TradePanel = ({ currentPrice, marketId }: TradePanelProps) => {
  const { publicKey } = useWallet();
  const openPositionMutation = useOpenPosition();
  const createLimitOrderMutation = useCreateLimitOrder();
  const { data: tokenBalance } = useUserTokenBalance();
  const { data: minFee } = useMinExecutionFee();

  const [orderMode, setOrderMode] = useState<OrderMode>("market");
  const [position, setPosition] = useState<"long" | "short">("long");
  const [collateral, setCollateral] = useState("");
  const [leverage, setLeverage] = useState([10]);
  const [triggerPrice, setTriggerPrice] = useState("");

  const isPending = openPositionMutation.isPending || createLimitOrderMutation.isPending;

  const calculatePositionSize = () => {
    const collateralAmount = parseFloat(collateral) || 0;
    return collateralAmount * leverage[0];
  };

  const calculateLiquidationPrice = () => {
    const collateralAmount = parseFloat(collateral) || 0;
    if (collateralAmount === 0) return 0;

    const maintenanceMargin = 0.5; // 0.5% maintenance margin
    const entryPrice = orderMode === "limit" ? parseFloat(triggerPrice) || currentPrice : currentPrice;

    if (position === "long") {
      return entryPrice * (1 - 1 / leverage[0] + maintenanceMargin / 100);
    }
    return entryPrice * (1 + 1 / leverage[0] - maintenanceMargin / 100);
  };

  const handleTrade = async (positionType: "long" | "short") => {
    setPosition(positionType);

    if (!publicKey) {
      toast.error("Please connect your wallet first");
      return;
    }

    if (!collateral || parseFloat(collateral) <= 0) {
      toast.error("Please enter a valid collateral amount");
      return;
    }

    const collateralAmount = parseFloat(collateral);

    if (tokenBalance && collateralAmount > tokenBalance.formatted) {
      toast.error("Insufficient balance", {
        description: `You have ${tokenBalance.formatted.toFixed(2)} USD available`,
      });
      return;
    }

    if (leverage[0] < 5 || leverage[0] > 20) {
      toast.error("Leverage must be between 5x and 20x");
      return;
    }

    // Market order
    if (orderMode === "market") {
      openPositionMutation.mutate(
        {
          marketId,
          collateral: collateralAmount,
          leverage: leverage[0],
          isLong: positionType === "long",
        },
        {
          onSuccess: () => {
            setCollateral("");
            setLeverage([10]);
          },
        }
      );
      return;
    }

    // Limit order
    const trigger = parseFloat(triggerPrice);
    if (!trigger || trigger <= 0) {
      toast.error("Please enter a valid trigger price");
      return;
    }

    const executionFee = minFee?.formatted || 0.1;

    createLimitOrderMutation.mutate(
      {
        marketId,
        triggerPrice: trigger,
        collateral: collateralAmount,
        leverage: leverage[0],
        isLong: positionType === "long",
        executionFee,
      },
      {
        onSuccess: () => {
          setCollateral("");
          setLeverage([10]);
          setTriggerPrice("");
        },
      }
    );
  };

  const entryPrice = orderMode === "limit" ? parseFloat(triggerPrice) || currentPrice : currentPrice;

  return (
    <Card className="h-full">
      <CardHeader className="pb-4">
        <CardTitle>Open Position</CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Order Mode Toggle */}
        <div className="flex gap-2 p-1 bg-secondary rounded-lg">
          <button
            onClick={() => setOrderMode("market")}
            className={`flex-1 py-2 text-sm font-medium rounded transition-colors ${
              orderMode === "market"
                ? "bg-background shadow-sm"
                : "text-muted-foreground hover:text-foreground"
            }`}
            disabled={isPending}
          >
            Market
          </button>
          <button
            onClick={() => setOrderMode("limit")}
            className={`flex-1 py-2 text-sm font-medium rounded transition-colors ${
              orderMode === "limit"
                ? "bg-background shadow-sm"
                : "text-muted-foreground hover:text-foreground"
            }`}
            disabled={isPending}
          >
            Limit
          </button>
        </div>

        <div className="p-3 bg-secondary rounded-lg">
          <div className="text-sm text-muted-foreground mb-1">Market Price</div>
          <div className="text-xl font-bold">${currentPrice.toLocaleString()}</div>
        </div>

        {/* Trigger Price (Limit orders only) */}
        {orderMode === "limit" && (
          <div>
            <Label htmlFor="triggerPrice">Trigger Price</Label>
            <Input
              id="triggerPrice"
              type="number"
              placeholder={currentPrice.toFixed(4)}
              value={triggerPrice}
              onChange={(e) => setTriggerPrice(e.target.value)}
              disabled={isPending}
              className="mt-2"
            />
            <p className="text-xs text-muted-foreground mt-1">
              Order executes when price reaches this level
            </p>
          </div>
        )}

        <div>
          <div className="flex justify-between items-center mb-2">
            <Label htmlFor="collateral">Collateral (USD)</Label>
            {tokenBalance && (
              <span className="text-xs text-muted-foreground">
                Balance: ${tokenBalance.formatted.toFixed(2)}
              </span>
            )}
          </div>
          <Input
            id="collateral"
            type="number"
            placeholder="0.00"
            value={collateral}
            onChange={(e) => setCollateral(e.target.value)}
            disabled={isPending}
          />
          {tokenBalance && (
            <Button
              variant="link"
              size="sm"
              className="h-auto p-0 mt-1 text-xs"
              onClick={() => setCollateral(tokenBalance.formatted.toString())}
              disabled={isPending}
            >
              Max
            </Button>
          )}
        </div>

        <div>
          <div className="flex justify-between mb-2">
            <Label>Leverage</Label>
            <span className="text-sm font-medium">{leverage[0]}x</span>
          </div>
          <Slider
            value={leverage}
            onValueChange={setLeverage}
            min={5}
            max={20}
            step={1}
            className="mb-2"
            disabled={isPending}
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>5x</span>
            <span>20x (Max)</span>
          </div>
        </div>

        <div className="space-y-2 p-4 bg-secondary rounded-lg">
          <div className="flex justify-between text-sm">
            <span className="text-muted-foreground">Position Size</span>
            <span className="font-medium">{calculatePositionSize().toFixed(2)} USD</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-muted-foreground">
              {orderMode === "limit" ? "Trigger Price" : "Entry Price"}
            </span>
            <span className="font-medium">${entryPrice.toLocaleString()}</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-muted-foreground">Liquidation Price</span>
            <span className="font-medium text-danger">
              ${calculateLiquidationPrice().toLocaleString(undefined, {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              })}
            </span>
          </div>
          {orderMode === "limit" && minFee && (
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Execution Fee</span>
              <span className="font-medium">${minFee.formatted.toFixed(2)}</span>
            </div>
          )}
        </div>

        <div className="grid grid-cols-2 gap-3">
          <Button
            onClick={() => handleTrade("long")}
            className="bg-success hover:bg-success/90 text-success-foreground"
            disabled={!publicKey || isPending}
          >
            {isPending && position === "long" ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin mr-2" />
                {orderMode === "limit" ? "Placing..." : "Opening..."}
              </>
            ) : orderMode === "limit" ? (
              "Long Limit"
            ) : (
              "Long"
            )}
          </Button>
          <Button
            onClick={() => handleTrade("short")}
            className="bg-danger hover:bg-danger/90 text-danger-foreground"
            disabled={!publicKey || isPending}
          >
            {isPending && position === "short" ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin mr-2" />
                {orderMode === "limit" ? "Placing..." : "Opening..."}
              </>
            ) : orderMode === "limit" ? (
              "Short Limit"
            ) : (
              "Short"
            )}
          </Button>
        </div>
        {!publicKey && (
          <p className="text-sm text-center text-muted-foreground">Connect wallet to trade</p>
        )}
      </CardContent>
    </Card>
  );
};

export default TradePanel;
