import { Button } from "@/components/ui/button";
import { Loader2, X } from "lucide-react";
import {
  useUserOrders,
  useCancelOrder,
  FormattedOrder,
} from "@/hooks/usePositionManager";
import { useWallet } from "@/hooks/useWallet";

const ORDER_TYPE_COLORS: Record<string, string> = {
  Limit: "bg-purple-500/10 text-purple-500",
  StopLoss: "bg-orange-500/10 text-orange-500",
  TakeProfit: "bg-green-500/10 text-green-500",
};

const OrderCard = ({
  order,
  onCancel,
  isCanceling,
}: {
  order: FormattedOrder;
  onCancel: (orderId: bigint) => void;
  isCanceling: boolean;
}) => {
  const isLimitOrder = order.orderTypeString === "Limit";

  return (
    <div className="p-4 border border-border rounded-lg space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <span className="font-semibold">{order.marketName}</span>
          <span
            className={`px-2 py-1 text-xs font-medium rounded ${
              ORDER_TYPE_COLORS[order.orderTypeString] || "bg-secondary"
            }`}
          >
            {order.orderTypeString === "StopLoss"
              ? "Stop Loss"
              : order.orderTypeString === "TakeProfit"
              ? "Take Profit"
              : "Limit"}
          </span>
          {isLimitOrder && (
            <span
              className={`px-2 py-1 text-xs font-medium rounded ${
                order.isLong
                  ? "bg-success/10 text-success"
                  : "bg-danger/10 text-danger"
              }`}
            >
              {order.isLong ? "LONG" : "SHORT"} {order.leverage}x
            </span>
          )}
          {order.isExpired && (
            <span className="px-2 py-1 text-xs font-medium rounded bg-muted text-muted-foreground">
              Expired
            </span>
          )}
        </div>
        <Button
          variant="ghost"
          size="icon"
          onClick={() => onCancel(order.id)}
          disabled={isCanceling}
        >
          {isCanceling ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <X className="h-4 w-4" />
          )}
        </Button>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
        <div>
          <div className="text-muted-foreground mb-1">Trigger Price</div>
          <div className="font-medium">
            $
            {order.triggerPrice.toLocaleString(undefined, {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </div>
        </div>
        {isLimitOrder ? (
          <>
            <div>
              <div className="text-muted-foreground mb-1">Collateral</div>
              <div className="font-medium">
                $
                {order.collateral.toLocaleString(undefined, {
                  minimumFractionDigits: 2,
                  maximumFractionDigits: 2,
                })}
              </div>
            </div>
            <div>
              <div className="text-muted-foreground mb-1">Size</div>
              <div className="font-medium">
                $
                {order.size.toLocaleString(undefined, {
                  minimumFractionDigits: 2,
                  maximumFractionDigits: 2,
                })}
              </div>
            </div>
          </>
        ) : (
          <>
            <div>
              <div className="text-muted-foreground mb-1">Close %</div>
              <div className="font-medium">{order.closePercentage}%</div>
            </div>
            <div>
              <div className="text-muted-foreground mb-1">Position ID</div>
              <div className="font-medium">#{order.positionId.toString()}</div>
            </div>
          </>
        )}
        <div>
          <div className="text-muted-foreground mb-1">Exec Fee</div>
          <div className="font-medium">
            $
            {order.executionFee.toLocaleString(undefined, {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </div>
        </div>
      </div>
    </div>
  );
};

const OrdersTab = () => {
  const { publicKey } = useWallet();
  const { data: orders = [], isLoading, isError } = useUserOrders();
  const cancelOrderMutation = useCancelOrder();

  const handleCancelOrder = (orderId: bigint) => {
    cancelOrderMutation.mutate(orderId);
  };

  if (!publicKey) {
    return (
      <div className="text-center py-12 text-muted-foreground">
        Connect wallet to view orders
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="text-center py-12">
        <Loader2 className="h-8 w-8 animate-spin mx-auto text-muted-foreground" />
      </div>
    );
  }

  if (isError) {
    return (
      <div className="text-center py-12 text-danger">Error loading orders</div>
    );
  }

  if (orders.length === 0) {
    return (
      <div className="text-center py-12 text-muted-foreground">
        No active orders
      </div>
    );
  }

  // Group orders by type
  const limitOrders = orders.filter((o) => o.orderTypeString === "Limit");
  const slOrders = orders.filter((o) => o.orderTypeString === "StopLoss");
  const tpOrders = orders.filter((o) => o.orderTypeString === "TakeProfit");

  return (
    <div className="space-y-6">
      {limitOrders.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-medium text-muted-foreground">
            Limit Orders ({limitOrders.length})
          </h3>
          {limitOrders.map((order) => (
            <OrderCard
              key={order.id.toString()}
              order={order}
              onCancel={handleCancelOrder}
              isCanceling={cancelOrderMutation.isPending}
            />
          ))}
        </div>
      )}

      {slOrders.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-medium text-muted-foreground">
            Stop Loss Orders ({slOrders.length})
          </h3>
          {slOrders.map((order) => (
            <OrderCard
              key={order.id.toString()}
              order={order}
              onCancel={handleCancelOrder}
              isCanceling={cancelOrderMutation.isPending}
            />
          ))}
        </div>
      )}

      {tpOrders.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-medium text-muted-foreground">
            Take Profit Orders ({tpOrders.length})
          </h3>
          {tpOrders.map((order) => (
            <OrderCard
              key={order.id.toString()}
              order={order}
              onCancel={handleCancelOrder}
              isCanceling={cancelOrderMutation.isPending}
            />
          ))}
        </div>
      )}
    </div>
  );
};

export default OrdersTab;
