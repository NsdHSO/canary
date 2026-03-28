import React from 'react';
import type { DtcResponse } from '../types';

interface DtcTableProps {
  dtcs: DtcResponse[];
  onClear?: (code: string) => void;
  loading?: boolean;
}

const severityColors: Record<string, string> = {
  high: 'text-red-400 bg-red-900/30',
  medium: 'text-yellow-400 bg-yellow-900/30',
  low: 'text-green-400 bg-green-900/30',
};

export const DtcTable: React.FC<DtcTableProps> = ({ dtcs, onClear, loading }) => {
  if (loading) {
    return (
      <div className="animate-pulse space-y-3">
        {[1, 2, 3].map((i) => (
          <div key={i} className="h-12 bg-gray-700 rounded" />
        ))}
      </div>
    );
  }

  if (dtcs.length === 0) {
    return (
      <div className="text-center text-gray-400 py-8">
        No DTCs found. Vehicle is healthy.
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm text-left">
        <thead className="text-xs text-gray-400 uppercase bg-gray-800">
          <tr>
            <th className="px-4 py-3">Code</th>
            <th className="px-4 py-3">Description</th>
            <th className="px-4 py-3">System</th>
            <th className="px-4 py-3">Severity</th>
            <th className="px-4 py-3">Status</th>
            {onClear && <th className="px-4 py-3">Actions</th>}
          </tr>
        </thead>
        <tbody>
          {dtcs.map((dtc) => (
            <tr
              key={dtc.code}
              className="border-b border-gray-700 hover:bg-gray-800/50"
            >
              <td className="px-4 py-3 font-mono font-bold text-canary-400">
                {dtc.code}
              </td>
              <td className="px-4 py-3">{dtc.description}</td>
              <td className="px-4 py-3 text-gray-400">{dtc.system}</td>
              <td className="px-4 py-3">
                <span
                  className={`px-2 py-1 rounded text-xs font-medium ${
                    severityColors[dtc.severity] || 'text-gray-400'
                  }`}
                >
                  {dtc.severity.toUpperCase()}
                </span>
              </td>
              <td className="px-4 py-3 font-mono text-xs">
                0x{dtc.status.toString(16).toUpperCase().padStart(2, '0')}
              </td>
              {onClear && (
                <td className="px-4 py-3">
                  <button
                    onClick={() => onClear(dtc.code)}
                    className="px-3 py-1 text-xs bg-red-600 hover:bg-red-700 rounded transition-colors"
                  >
                    Clear
                  </button>
                </td>
              )}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};
