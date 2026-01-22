import { useParams, useNavigate } from "react-router-dom";
import Header from "@/components/Header";
import TradeChart from "@/components/TradeChart";
import TradePanel from "@/components/TradePanel";
import PositionsPanel from "@/components/PositionsPanel";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { TRADING_PAIRS } from "@/types/market";
import { usePrices } from "@/contexts/PriceContext";

// Map trading pair symbols to market IDs (matches contract)
const SYMBOL_TO_MARKET_ID: Record<string, number> = {
  XLMUSDT: 0,
  BTCUSDT: 1,
  ETHUSDT: 2,
};

const Trade = () => {
  const { pair } = useParams<{ pair: string }>();
  const navigate = useNavigate();
  const { getPrice } = usePrices();

  // Find the current trading pair
  const currentPair = TRADING_PAIRS.find((p) => p.displaySymbol === pair) || TRADING_PAIRS[0];

  // Get live price data
  const priceData = getPrice(currentPair.symbol);
  const currentPrice = priceData?.price || 0;

  return (
    <div className="min-h-screen bg-background">
      <Header />

      <main className="container mx-auto px-4 py-6">
        <div className="mb-6">
          <Select value={currentPair.displaySymbol} onValueChange={(value) => navigate(`/trade/${value}`)}>
            <SelectTrigger className="w-[280px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {TRADING_PAIRS.map((tradingPair) => (
                <SelectItem key={tradingPair.displaySymbol} value={tradingPair.displaySymbol}>
                  <div className="flex items-center justify-between gap-4">
                    <span>{tradingPair.name}</span>
                    <span className="text-muted-foreground">{tradingPair.displaySymbol}</span>
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 mb-6">
          <div className="lg:col-span-3">
            <TradePanel asset={currentPair.displaySymbol} currentPrice={currentPrice} marketId={SYMBOL_TO_MARKET_ID[currentPair.symbol] ?? 0} />
          </div>
          <div className="lg:col-span-9">
            <TradeChart symbol={currentPair.symbol} marketId={SYMBOL_TO_MARKET_ID[currentPair.symbol] ?? 0} />
          </div>
        </div>

        <PositionsPanel />
      </main>
    </div>
  );
};

export default Trade;
