import React from 'react';
import type { LiveDataPoint } from '../types';

interface LiveGaugeProps {
  data: LiveDataPoint | undefined;
  title: string;
  color?: string;
}

export const LiveGauge: React.FC<LiveGaugeProps> = ({
  data,
  title,
  color = '#f59e0b',
}) => {
  const value = data?.value ?? 0;
  const min = data?.min ?? 0;
  const max = data?.max ?? 100;
  const unit = data?.unit ?? '';
  const percentage = Math.min(100, Math.max(0, ((value - min) / (max - min)) * 100));

  return (
    <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
      <h3 className="text-sm font-medium text-gray-400 mb-2">{title}</h3>
      <div className="flex items-end gap-2">
        <span className="text-3xl font-bold" style={{ color }}>
          {value.toFixed(1)}
        </span>
        <span className="text-gray-500 text-sm mb-1">{unit}</span>
      </div>
      <div className="mt-3 w-full bg-gray-700 rounded-full h-2">
        <div
          className="h-2 rounded-full transition-all duration-150"
          style={{
            width: `${percentage}%`,
            backgroundColor: color,
          }}
        />
      </div>
      <div className="flex justify-between text-xs text-gray-500 mt-1">
        <span>{min}</span>
        <span>{max}</span>
      </div>
    </div>
  );
};
