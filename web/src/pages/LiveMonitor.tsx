import React from 'react';
import { LiveGauge } from '../components/LiveGauge';
import { LiveChart } from '../components/LiveChart';
import { ConnectionStatus } from '../components/ConnectionStatus';
import { useLiveData } from '../hooks/useLiveData';

const ECU_ID = 0x7e0;
const PIDS = [0x0c, 0x0d, 0x05, 0x0b, 0x04, 0x11];

const pidNames: Record<number, { name: string; unit: string; color: string }> = {
  0x0c: { name: 'Engine RPM', unit: 'rpm', color: '#f59e0b' },
  0x0d: { name: 'Vehicle Speed', unit: 'km/h', color: '#3b82f6' },
  0x05: { name: 'Coolant Temp', unit: 'C', color: '#ef4444' },
  0x0b: { name: 'Intake MAP', unit: 'kPa', color: '#10b981' },
  0x04: { name: 'Engine Load', unit: '%', color: '#8b5cf6' },
  0x11: { name: 'Throttle', unit: '%', color: '#ec4899' },
};

export const LiveMonitor: React.FC = () => {
  const { data, connected, latestValues } = useLiveData(ECU_ID, PIDS);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Live ECU Monitor</h2>
          <p className="text-gray-400 text-sm">
            ECU 0x{ECU_ID.toString(16).toUpperCase()} - Real-time data
          </p>
        </div>
        <ConnectionStatus connected={connected} />
      </div>

      {/* Gauge Grid */}
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
        {PIDS.map((pid) => {
          const info = pidNames[pid];
          return (
            <LiveGauge
              key={pid}
              data={latestValues[pid]}
              title={info?.name || `PID 0x${pid.toString(16)}`}
              color={info?.color}
            />
          );
        })}
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {PIDS.slice(0, 4).map((pid) => {
          const info = pidNames[pid];
          const key = `pid_${pid}`;
          return (
            <LiveChart
              key={pid}
              data={data[key] || []}
              title={info?.name || `PID 0x${pid.toString(16)}`}
              color={info?.color}
              unit={info?.unit}
            />
          );
        })}
      </div>
    </div>
  );
};
