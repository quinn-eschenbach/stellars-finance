import { useState, useMemo } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { CandlestickChart, PriceLine } from "./CandlestickChart";
import { useCandleData } from "@/hooks/useCandleData";
import { TimeFrame } from "@/types/market";
import { usePrices } from "@/contexts/PriceContext";
import { LineStyle } from "lightweight-charts";
import {
  useUserPositions,
  useUserOrders,
  FormattedPosition,
  FormattedOrder,
} from "@/hooks/usePositionManager";

interface TradeChartProps {
  symbol: string;
  marketId?: number;
}

const TIMEFRAMES: { label: string; value: TimeFrame }[] = [
  { label: "1H", value: "1h" },
  { label: "4H", value: "4h" },
  { label: "1D", value: "1d" },
  { label: "1W", value: "1w" },
];

// Color scheme for price lines
const PRICE_LINE_COLORS = {
  entry: "#3b82f6", // blue
  liquidation: "#ef4444", // red
  stopLoss: "#f97316", // orange
  takeProfit: "#22c55e", // green
  limitOrder: "#8b5cf6", // purple
};

const buildPriceLines = (
  positions: FormattedPosition[],
  orders: FormattedOrder[],
  marketId: number
): PriceLine[] => {
  const lines: PriceLine[] = [];

  // Add position lines (entry and liquidation)
  // Note: For MVP, all positions are market 0 (XLM-PERP)
  positions.forEach((pos) => {
    // Entry price line
    lines.push({
      id: `entry-${pos.id}`,
      price: pos.entryPrice,
      color: PRICE_LINE_COLORS.entry,
      title: `Entry $${pos.entryPrice.toFixed(2)}`,
      lineStyle: LineStyle.Dashed,
      lineWidth: 1,
    });

    // Liquidation price line
    lines.push({
      id: `liq-${pos.id}`,
      price: pos.liquidationPrice,
      color: PRICE_LINE_COLORS.liquidation,
      title: `Liq $${pos.liquidationPrice.toFixed(2)}`,
      lineStyle: LineStyle.Dotted,
      lineWidth: 1,
    });
  });

  // Add order lines (SL, TP, Limit)
  orders
    .filter((order) => order.marketId === marketId && !order.isExpired)
    .forEach((order) => {
      let color = PRICE_LINE_COLORS.limitOrder;
      let prefix = "Limit";

      if (order.orderTypeString === "StopLoss") {
        color = PRICE_LINE_COLORS.stopLoss;
        prefix = "SL";
      } else if (order.orderTypeString === "TakeProfit") {
        color = PRICE_LINE_COLORS.takeProfit;
        prefix = "TP";
      }

      lines.push({
        id: `order-${order.id}`,
        price: order.triggerPrice,
        color,
        title: `${prefix} $${order.triggerPrice.toFixed(2)}`,
        lineStyle: order.orderTypeString === "Limit" ? LineStyle.Dashed : LineStyle.Solid,
        lineWidth: 1,
      });
    });

  return lines;
};

const TradeChart = ({ symbol, marketId = 0 }: TradeChartProps) => {
  const [timeframe, setTimeframe] = useState<TimeFrame>("1h");
  const { candles, isLoading, error } = useCandleData(symbol, timeframe);
  const { getPrice } = usePrices();

  // Fetch positions and orders for price lines
  const { data: positions = [] } = useUserPositions();
  const { data: orders = [] } = useUserOrders();

  const priceData = getPrice(symbol);
  const currentPrice = priceData?.price || 0;
  const priceChangePercent = priceData?.change24h || 0;

  // Build price lines from positions and orders
  const priceLines = useMemo(
    () => buildPriceLines(positions, orders, marketId),
    [positions, orders, marketId]
  );

  return (
    <Card className="h-full">
      <CardContent className="p-6">
        <div className="flex items-center justify-between mb-4">
          <div>
            <div className="text-3xl font-bold">
              ${currentPrice.toLocaleString(undefined, {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              })}
            </div>
            <div
              className={`text-sm font-medium ${
                priceChangePercent >= 0 ? "text-success" : "text-danger"
              }`}
            >
              {priceChangePercent >= 0 ? "+" : ""}
              {priceChangePercent.toFixed(2)}% (24h)
            </div>
          </div>
          <div className="flex gap-2">
            {TIMEFRAMES.map((tf) => (
              <button
                key={tf.value}
                onClick={() => setTimeframe(tf.value)}
                className={`px-3 py-1 text-sm rounded transition-colors ${
                  timeframe === tf.value
                    ? "bg-primary text-primary-foreground"
                    : "bg-secondary hover:bg-accent"
                }`}
              >
                {tf.label}
              </button>
            ))}
          </div>
        </div>

        {isLoading ? (
          <div className="h-[400px] flex items-center justify-center">
            <div className="text-muted-foreground">Loading chart...</div>
          </div>
        ) : error ? (
          <div className="h-[400px] flex items-center justify-center">
            <div className="text-danger">Error loading chart: {error}</div>
          </div>
        ) : (
          <CandlestickChart candles={candles} height={400} priceLines={priceLines} />
        )}
      </CardContent>
    </Card>
  );
};

export default TradeChart;
