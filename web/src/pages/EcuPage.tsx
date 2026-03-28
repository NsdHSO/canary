import React, { useEffect, useState } from 'react';
import { listEcus } from '../services/api';
import type { EcuInfo } from '../types';

export const EcuPage: React.FC = () => {
  const [ecus, setEcus] = useState<EcuInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState('');

  useEffect(() => {
    const fetch = async () => {
      setLoading(true);
      try {
        const data = await listEcus(filter || undefined);
        setEcus(data.items);
      } catch (e) {
        console.error('Failed to fetch ECUs:', e);
      } finally {
        setLoading(false);
      }
    };
    fetch();
  }, [filter]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">ECU Database</h2>
          <p className="text-gray-400 text-sm">{ecus.length} ECUs available</p>
        </div>
        <input
          type="text"
          placeholder="Filter by manufacturer..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-sm focus:outline-none focus:border-canary-500"
        />
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {loading ? (
          [1, 2, 3].map((i) => (
            <div key={i} className="bg-gray-800 rounded-lg p-4 border border-gray-700 animate-pulse h-36" />
          ))
        ) : (
          ecus.map((ecu) => (
            <div
              key={ecu.id}
              className="bg-gray-800 rounded-lg p-4 border border-gray-700 hover:border-canary-600 transition-colors"
            >
              <div className="flex justify-between items-start">
                <div>
                  <h3 className="font-semibold">{ecu.manufacturer} {ecu.model}</h3>
                  <p className="text-sm text-gray-400">{ecu.year_range}</p>
                </div>
                <span className="px-2 py-1 bg-canary-900/30 text-canary-400 rounded text-xs font-mono">
                  {ecu.ecu_type}
                </span>
              </div>
              <div className="mt-3 text-sm text-gray-400">
                <p>CAN ID: <span className="font-mono text-white">0x{ecu.can_id.toString(16).toUpperCase()}</span></p>
                <div className="flex gap-1 mt-2">
                  {ecu.protocols.map((p) => (
                    <span key={p} className="px-2 py-0.5 bg-gray-700 rounded text-xs">{p}</span>
                  ))}
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
