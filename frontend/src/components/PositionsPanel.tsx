import { useState } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { X, Loader2, ChevronDown, Target, Shield } from "lucide-react";
import {
  useUserPositions,
  useClosePosition,
  usePositionOrders,
  FormattedPosition,
} from "@/hooks/usePositionManager";
import { useWallet } from "@/hooks/useWallet";
import { usePrices } from "@/contexts/PriceContext";
import OrdersTab from "./OrdersTab";
import SetOrderDialog from "./SetOrderDialog";

const MARKET_NAMES: Record<number, string> = {
  0: "XLM-PERP",
  1: "BTC-PERP",
  2: "ETH-PERP",
};

const MARKET_TO_BINANCE_SYMBOL: Record<number, string> = {
  0: "XLMUSDT",
  1: "BTCUSDT",
  2: "ETHUSDT",
};

const PositionOrderBadges = ({ positionId }: { positionId: bigint }) => {
  const { data: orders = [] } = usePositionOrders(positionId);

  const slOrder = orders.find((o) => o.orderTypeString === "StopLoss");
  const tpOrder = orders.find((o) => o.orderTypeString === "TakeProfit");

  if (!slOrder && !tpOrder) return null;

  return (
    <div className="flex gap-2 flex-wrap">
      {slOrder && (
        <span className="px-2 py-0.5 text-xs font-medium rounded bg-orange-500/10 text-orange-500">
          SL: ${slOrder.triggerPrice.toFixed(2)}
        </span>
      )}
      {tpOrder && (
        <span className="px-2 py-0.5 text-xs font-medium rounded bg-green-500/10 text-green-500">
          TP: ${tpOrder.triggerPrice.toFixed(2)}
        </span>
      )}
    </div>
  );
};

interface PositionCardProps {
  position: FormattedPosition;
  onClose: (positionId: bigint) => void;
  isClosing: boolean;
}

const PositionCard = ({ position, onClose, isClosing }: PositionCardProps) => {
  const [orderDialogType, setOrderDialogType] = useState<"stopLoss" | "takeProfit" | null>(null);
  const { getPrice } = usePrices();

  // Calculate PnL using real-time price
  const binanceSymbol = MARKET_TO_BINANCE_SYMBOL[position.marketId];
  const priceData = getPrice(binanceSymbol);
  const currentPrice = priceData?.price ?? position.entryPrice;

  // PnL = (currentPrice - entryPrice) / entryPrice * size * direction
  const priceDiff = currentPrice - position.entryPrice;
  const direction = position.isLong ? 1 : -1;
  const pnlAmount = (priceDiff / position.entryPrice) * position.size * direction;
  const pnlPercent = (priceDiff / position.entryPrice) * 100 * direction;
  const isPnlPositive = pnlAmount >= 0;

  return (
    <>
      <div className="p-4 border border-border rounded-lg space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="font-semibold">{MARKET_NAMES[position.marketId] || "UNKNOWN"}</span>
            <span
              className={`px-2 py-1 text-xs font-medium rounded ${
                position.isLong ? "bg-success/10 text-success" : "bg-danger/10 text-danger"
              }`}
            >
              {position.isLong ? "LONG" : "SHORT"} {position.leverage.toFixed(1)}x
            </span>
            <PositionOrderBadges positionId={position.id} />
          </div>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => onClose(position.id)}
            disabled={isClosing}
          >
            <X className="h-4 w-4" />
          </Button>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
          <div>
            <div className="text-muted-foreground mb-1">Size</div>
            <div className="font-medium">
              ${position.size.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
            </div>
          </div>
          <div>
            <div className="text-muted-foreground mb-1">Collateral</div>
            <div className="font-medium">
              ${position.collateral.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
            </div>
          </div>
          <div>
            <div className="text-muted-foreground mb-1">Entry Price</div>
            <div className="font-medium">
              ${position.entryPrice.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
            </div>
          </div>
          <div>
            <div className="text-muted-foreground mb-1">Liquidation</div>
            <div className="font-medium text-danger">
              ${position.liquidationPrice.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
            </div>
          </div>
        </div>

        <div className="flex items-center justify-between pt-2 border-t border-border">
          <div>
            <span className="text-sm text-muted-foreground">PnL:</span>
            <span className={`font-semibold ${isPnlPositive ? 'text-success' : 'text-danger'}`}>
              {' '}{isPnlPositive ? '+' : ''}${pnlAmount.toFixed(2)} ({isPnlPositive ? '+' : ''}{pnlPercent.toFixed(2)}%)
            </span>
          </div>
          <div className="flex gap-2">
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button size="sm" variant="outline">
                  SL/TP <ChevronDown className="h-4 w-4 ml-1" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={() => setOrderDialogType("stopLoss")}>
                  <Shield className="h-4 w-4 mr-2 text-orange-500" />
                  Add Stop Loss
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => setOrderDialogType("takeProfit")}>
                  <Target className="h-4 w-4 mr-2 text-green-500" />
                  Add Take Profit
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
            <Button
              size="sm"
              variant="outline"
              onClick={() => onClose(position.id)}
              disabled={isClosing}
            >
              {isClosing ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin mr-2" />
                  Closing...
                </>
              ) : (
                "Close Position"
              )}
            </Button>
          </div>
        </div>
      </div>

      {orderDialogType && (
        <SetOrderDialog
          position={position}
          orderType={orderDialogType}
          open={true}
          onOpenChange={(open) => {
            if (!open) setOrderDialogType(null);
          }}
        />
      )}
    </>
  );
};

const PositionsPanel = () => {
  const { publicKey } = useWallet();
  const { data: positions = [], isLoading, isError } = useUserPositions();
  const closePositionMutation = useClosePosition();

  const handleClosePosition = (positionId: bigint) => {
    closePositionMutation.mutate(positionId);
  };

  return (
    <Card>
      <CardContent className="p-6">
        <Tabs defaultValue="positions" className="w-full">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="positions">Positions</TabsTrigger>
            <TabsTrigger value="orders">Orders</TabsTrigger>
            <TabsTrigger value="history">History</TabsTrigger>
          </TabsList>

          <TabsContent value="positions" className="mt-4">
            <div className="space-y-4">
              {!publicKey ? (
                <div className="text-center py-12 text-muted-foreground">
                  Connect wallet to view positions
                </div>
              ) : isLoading ? (
                <div className="text-center py-12">
                  <Loader2 className="h-8 w-8 animate-spin mx-auto text-muted-foreground" />
                </div>
              ) : isError ? (
                <div className="text-center py-12 text-danger">Error loading positions</div>
              ) : positions.length > 0 ? (
                positions.map((position) => (
                  <PositionCard
                    key={position.id.toString()}
                    position={position}
                    onClose={handleClosePosition}
                    isClosing={closePositionMutation.isPending}
                  />
                ))
              ) : (
                <div className="text-center py-12 text-muted-foreground">No open positions</div>
              )}
            </div>
          </TabsContent>

          <TabsContent value="orders" className="mt-4">
            <OrdersTab />
          </TabsContent>

          <TabsContent value="history" className="mt-4">
            <div className="space-y-3">
              <div className="text-center py-12 text-muted-foreground">
                <p className="mb-2">Trade history not yet implemented</p>
                <p className="text-sm">
                  Requires event indexing or backend service to track closed positions
                </p>
              </div>
            </div>
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  );
};

export default PositionsPanel;
