import React from 'react';

interface ConnectionStatusProps {
  connected: boolean;
  latency?: number;
}

export const ConnectionStatus: React.FC<ConnectionStatusProps> = ({
  connected,
  latency,
}) => {
  return (
    <div className="flex items-center gap-2 text-sm">
      <div
        className={`w-2.5 h-2.5 rounded-full ${
          connected ? 'bg-green-500 animate-pulse' : 'bg-red-500'
        }`}
      />
      <span className={connected ? 'text-green-400' : 'text-red-400'}>
        {connected ? 'Connected' : 'Disconnected'}
      </span>
      {latency !== undefined && connected && (
        <span className="text-gray-500">({latency}ms)</span>
      )}
    </div>
  );
};
