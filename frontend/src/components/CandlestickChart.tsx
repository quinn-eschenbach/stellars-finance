import { useEffect, useRef } from "react";
import { createChart, ColorType, CandlestickSeries, LineStyle } from "lightweight-charts";
import type { IChartApi, ISeriesApi, CandlestickData, IPriceLine } from "lightweight-charts";
import { Candle } from "@/types/market";

export interface PriceLine {
  id: string;
  price: number;
  color: string;
  title: string;
  lineStyle: LineStyle;
  lineWidth?: number;
}

interface CandlestickChartProps {
  candles: Candle[];
  height?: number;
  priceLines?: PriceLine[];
}

export const CandlestickChart = ({ candles, height = 400, priceLines }: CandlestickChartProps) => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candlestickSeriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const priceLinesRef = useRef<Map<string, IPriceLine>>(new Map());

  useEffect(() => {
    if (!chartContainerRef.current) return;

    // Create chart
    const chart = createChart(chartContainerRef.current, {
      width: chartContainerRef.current.clientWidth,
      height: height,
      layout: {
        background: { type: ColorType.Solid, color: "transparent" },
        textColor: "#71717a",
      },
      grid: {
        vertLines: { visible: false },
        horzLines: { visible: false },
      },
      crosshair: {
        mode: 1,
        vertLine: {
          color: "#71717a",
          width: 1,
          style: 3,
          labelBackgroundColor: "#27272a",
        },
        horzLine: {
          color: "#71717a",
          width: 1,
          style: 3,
          labelBackgroundColor: "#27272a",
        },
      },
      rightPriceScale: {
        borderColor: "#27272a",
        borderVisible: true,
      },
      timeScale: {
        borderColor: "#27272a",
        borderVisible: true,
        timeVisible: true,
        secondsVisible: false,
      },
    });

    chartRef.current = chart;

    // Create candlestick series
    const candlestickSeries = chart.addSeries(CandlestickSeries, {
      upColor: "#26a69a",
      downColor: "#ef5350",
      borderVisible: false,
      wickUpColor: "#26a69a",
      wickDownColor: "#ef5350",
    });

    candlestickSeriesRef.current = candlestickSeries;

    // Handle resize
    const handleResize = () => {
      if (chartContainerRef.current && chartRef.current) {
        chart.applyOptions({
          width: chartContainerRef.current.clientWidth,
        });
      }
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      // Clear price lines before removing chart
      priceLinesRef.current.clear();
      if (chartRef.current) {
        chart.remove();
      }
    };
  }, [height]);

  // Update candles
  useEffect(() => {
    if (!candlestickSeriesRef.current || candles.length === 0) return;

    const formattedCandles: CandlestickData[] = candles.map((candle) => ({
      time: candle.time as any,
      open: candle.open,
      high: candle.high,
      low: candle.low,
      close: candle.close,
    }));

    candlestickSeriesRef.current.setData(formattedCandles);

    // Fit content to view
    if (chartRef.current) {
      chartRef.current.timeScale().fitContent();
    }
  }, [candles]);

  // Update price lines
  useEffect(() => {
    if (!candlestickSeriesRef.current) return;

    const series = candlestickSeriesRef.current;
    const currentLines = priceLinesRef.current;
    const newLineIds = new Set(priceLines?.map((pl) => pl.id) || []);

    // Remove lines that no longer exist
    currentLines.forEach((line, id) => {
      if (!newLineIds.has(id)) {
        series.removePriceLine(line);
        currentLines.delete(id);
      }
    });

    // Update or create lines
    priceLines?.forEach((pl) => {
      const existing = currentLines.get(pl.id);
      if (existing) {
        // Update existing line
        existing.applyOptions({
          price: pl.price,
          title: pl.title,
          color: pl.color,
        });
      } else {
        // Create new line
        const newLine = series.createPriceLine({
          price: pl.price,
          color: pl.color,
          title: pl.title,
          lineStyle: pl.lineStyle,
          lineWidth: pl.lineWidth || 1,
          axisLabelVisible: true,
        });
        currentLines.set(pl.id, newLine);
      }
    });
  }, [priceLines]);

  return <div ref={chartContainerRef} className="w-full" />;
};
