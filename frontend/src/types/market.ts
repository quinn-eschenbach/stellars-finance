export interface Ticker {
  symbol: string;
  price: number;
  change24h: number;
  volume24h: string;
  high24h: number;
  low24h: number;
}

export interface Candle {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

export interface BinanceKline {
  t: number; // Kline start time
  T: number; // Kline close time
  s: string; // Symbol
  i: string; // Interval
  o: string; // Open price
  c: string; // Close price
  h: string; // High price
  l: string; // Low price
  v: string; // Base asset volume
  n: number; // Number of trades
  x: boolean; // Is this kline closed?
}

export interface BinanceTicker {
  e: string; // Event type
  E: number; // Event time
  s: string; // Symbol
  p: string; // Price change
  P: string; // Price change percent
  c: string; // Last price
  h: string; // High price
  l: string; // Low price
  v: string; // Total traded volume
}

export type TimeFrame = "1m" | "5m" | "15m" | "1h" | "4h" | "1d" | "1w";

export const TRADING_PAIRS = [
  { symbol: "XLMUSDT", displaySymbol: "XLM-USD", name: "Stellar" },
  { symbol: "BTCUSDT", displaySymbol: "BTC-USD", name: "Bitcoin" },
  { symbol: "ETHUSDT", displaySymbol: "ETH-USD", name: "Ethereum" },
] as const;
