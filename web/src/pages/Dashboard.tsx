import React, { useEffect, useState } from 'react';
import { LiveGauge } from '../components/LiveGauge';
import { LiveChart } from '../components/LiveChart';
import { ConnectionStatus } from '../components/ConnectionStatus';
import { useLiveData } from '../hooks/useLiveData';
import { healthCheck } from '../services/api';
import type { HealthResponse } from '../types';

const ECU_ID = 0x7e0;
const PIDS = [0x0c, 0x0d]; // RPM, Speed

export const Dashboard: React.FC = () => {
  const { data, connected, latestValues } = useLiveData(ECU_ID, PIDS);
  const [health, setHealth] = useState<HealthResponse | null>(null);

  useEffect(() => {
    healthCheck()
      .then(setHealth)
      .catch(() => setHealth(null));
  }, []);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Dashboard</h2>
          <p className="text-gray-400 text-sm">Real-time ECU monitoring</p>
        </div>
        <div className="flex items-center gap-4">
          <ConnectionStatus connected={connected} />
          {health && (
            <span className="text-xs text-gray-500">
              API v{health.version}
            </span>
          )}
        </div>
      </div>

      {/* Live Gauges */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <LiveGauge
          data={latestValues[0x0c]}
          title="Engine RPM"
          color="#f59e0b"
        />
        <LiveGauge
          data={latestValues[0x0d]}
          title="Vehicle Speed"
          color="#3b82f6"
        />
        <LiveGauge
          data={latestValues[0x05]}
          title="Coolant Temp"
          color="#ef4444"
        />
        <LiveGauge
          data={latestValues[0x0b]}
          title="Intake Pressure"
          color="#10b981"
        />
      </div>

      {/* Live Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <LiveChart
          data={data['pid_12'] || []}
          title="Engine RPM"
          color="#f59e0b"
          unit="rpm"
        />
        <LiveChart
          data={data['pid_13'] || []}
          title="Vehicle Speed"
          color="#3b82f6"
          unit="km/h"
        />
      </div>

      {/* Quick Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <p className="text-xs text-gray-400 uppercase">ECU</p>
          <p className="text-lg font-mono mt-1">0x{ECU_ID.toString(16).toUpperCase()}</p>
        </div>
        <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <p className="text-xs text-gray-400 uppercase">Data Points</p>
          <p className="text-lg font-mono mt-1">
            {Object.values(data).reduce((sum, arr) => sum + arr.length, 0)}
          </p>
        </div>
        <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <p className="text-xs text-gray-400 uppercase">Active PIDs</p>
          <p className="text-lg font-mono mt-1">{Object.keys(latestValues).length}</p>
        </div>
        <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <p className="text-xs text-gray-400 uppercase">Status</p>
          <p className={`text-lg mt-1 ${connected ? 'text-green-400' : 'text-red-400'}`}>
            {connected ? 'Online' : 'Offline'}
          </p>
        </div>
      </div>
    </div>
  );
};
