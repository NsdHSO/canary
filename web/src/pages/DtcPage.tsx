import React, { useCallback, useEffect, useState } from 'react';
import { DtcTable } from '../components/DtcTable';
import { readDtcs, clearDtcs } from '../services/api';
import type { DtcResponse } from '../types';

export const DtcPage: React.FC = () => {
  const [dtcs, setDtcs] = useState<DtcResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [clearing, setClearing] = useState(false);

  const fetchDtcs = useCallback(async () => {
    setLoading(true);
    try {
      const data = await readDtcs();
      setDtcs(data);
    } catch (e) {
      console.error('Failed to fetch DTCs:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchDtcs();
  }, [fetchDtcs]);

  const handleClear = async (code: string) => {
    setClearing(true);
    try {
      await clearDtcs({ ecu_id: 0x7e0, codes: [code] });
      setDtcs((prev) => prev.filter((d) => d.code !== code));
    } catch (e) {
      console.error('Failed to clear DTC:', e);
    } finally {
      setClearing(false);
    }
  };

  const handleClearAll = async () => {
    setClearing(true);
    try {
      await clearDtcs({ ecu_id: 0x7e0 });
      setDtcs([]);
    } catch (e) {
      console.error('Failed to clear all DTCs:', e);
    } finally {
      setClearing(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Diagnostic Trouble Codes</h2>
          <p className="text-gray-400 text-sm">
            {dtcs.length} DTC{dtcs.length !== 1 ? 's' : ''} found
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={fetchDtcs}
            disabled={loading}
            className="px-4 py-2 bg-canary-600 hover:bg-canary-700 rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
          >
            Refresh
          </button>
          <button
            onClick={handleClearAll}
            disabled={clearing || dtcs.length === 0}
            className="px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
          >
            Clear All
          </button>
        </div>
      </div>

      <div className="bg-gray-800 rounded-lg border border-gray-700">
        <DtcTable dtcs={dtcs} onClear={handleClear} loading={loading} />
      </div>
    </div>
  );
};
