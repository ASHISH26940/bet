"use client";

import { useEffect, useRef, useState } from "react";
import { Line } from "react-chartjs-2";
import {
  Chart as ChartJS,
  LineElement,
  PointElement,
  LinearScale,
  TimeScale,
  Title,
  Tooltip,
  Legend,
  Filler,
} from "chart.js";
import "chartjs-adapter-date-fns";

ChartJS.register(
  LineElement,
  PointElement,
  LinearScale,
  TimeScale,
  Title,
  Tooltip,
  Legend,
  Filler
);

export default function CryptoGraph() {
  const [dataPoints, setDataPoints] = useState<{ x: number; y: number }[]>([]);
  const [amount, setAmount] = useState<string>("");
  const [currentBalance, setCurrentBalance] = useState<number>(0);
  const [betBalance, setBetBalance] = useState<number>(0);

  const ws = useRef<WebSocket | null>(null);

  useEffect(() => {
    ws.current = new WebSocket("ws://localhost:8080/ws/");

    ws.current.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      if (msg.type === "price_update" && msg.usd_value) {
        setDataPoints((prev) => [
          ...prev.slice(-100),
          { x: Date.now(), y: msg.usd_value },
        ]);
      }

      if (msg.type === "cashout_result") {
        // Update balances when cashout result is received
        setCurrentBalance(parseFloat(msg.balance));
        setBetBalance(msg.usd_amount);
      }
    };

    return () => {
      ws.current?.close();
    };
  }, []);

  const startGame = () => {
    if (!amount) return;
    ws.current?.send(
      JSON.stringify({
        type: "start",
        amount,
        crypto: "sol",
      })
    );
  };

  const stopGame = () => {
    ws.current?.send(JSON.stringify({ type: "stop" }));
  };

  return (
    <div className="p-6">
      <div className="flex gap-4 mb-4 items-center">
        <input
          type="number"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="Enter USD amount"
          className="px-4 py-2 border rounded w-48"
        />
        <button
          className="bg-green-600 text-white px-4 py-2 rounded hover:bg-green-700"
          onClick={startGame}
        >
          Start
        </button>
        <button
          className="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
          onClick={stopGame}
        >
          Stop
        </button>
      </div>

      <div className="mb-4">
        <p>Current Balance: ${currentBalance.toFixed(2)}</p>
        <p>Betted Balance: ${betBalance.toFixed(2)}</p>
      </div>

      <Line
        data={{
          datasets: [
            {
              label: "USD Value Over Time",
              data: dataPoints,
              fill: true,
              borderColor: "#4ade80",
              backgroundColor: "rgba(74, 222, 128, 0.2)",
              tension: 0.3,
            },
          ],
        }}
        options={{
          responsive: true,
          animation: false,
          scales: {
            x: {
              type: "time",
              time: {
                unit: "second",
              },
              title: {
                display: true,
                text: "Time",
              },
            },
            y: {
              title: {
                display: true,
                text: "USD Value",
              },
            },
          },
        }}
      />
    </div>
  );
};
