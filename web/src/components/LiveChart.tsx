import React from 'react';
import { Line } from 'react-chartjs-2';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler,
} from 'chart.js';
import type { LiveDataPoint } from '../types';

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler
);

interface LiveChartProps {
  data: LiveDataPoint[];
  title: string;
  color?: string;
  unit?: string;
}

export const LiveChart: React.FC<LiveChartProps> = ({
  data,
  title,
  color = '#f59e0b',
  unit = '',
}) => {
  const chartData = {
    labels: data.map((p) => (p.timestamp_ms / 1000).toFixed(1)),
    datasets: [
      {
        label: `${title} (${unit})`,
        data: data.map((p) => p.value),
        borderColor: color,
        backgroundColor: `${color}20`,
        fill: true,
        tension: 0.3,
        pointRadius: 0,
        borderWidth: 2,
      },
    ],
  };

  const options = {
    responsive: true,
    maintainAspectRatio: false,
    animation: { duration: 0 },
    plugins: {
      legend: { display: false },
      title: {
        display: true,
        text: title,
        color: '#9ca3af',
        font: { size: 14 },
      },
    },
    scales: {
      x: {
        display: true,
        title: { display: true, text: 'Time (s)', color: '#6b7280' },
        ticks: { color: '#6b7280', maxTicksLimit: 10 },
        grid: { color: '#374151' },
      },
      y: {
        display: true,
        title: { display: true, text: unit, color: '#6b7280' },
        ticks: { color: '#6b7280' },
        grid: { color: '#374151' },
      },
    },
  };

  return (
    <div className="bg-gray-800 rounded-lg p-4 border border-gray-700 h-64">
      <Line data={chartData} options={options} />
    </div>
  );
};
