import { useCallback, useEffect, useRef, useState } from 'react';
import type { LiveDataPoint } from '../types';
import { wsClient } from '../services/websocket';

const MAX_DATA_POINTS = 200;

export interface LiveDataState {
  [key: string]: LiveDataPoint[];
}

export function useLiveData(ecuId: number, pids: number[]) {
  const [data, setData] = useState<LiveDataState>({});
  const [connected, setConnected] = useState(false);
  const [latestValues, setLatestValues] = useState<Record<number, LiveDataPoint>>({});
  const dataRef = useRef<LiveDataState>({});

  const handleLiveData = useCallback((point: LiveDataPoint) => {
    if (point.ecu_id !== ecuId) return;

    const key = `pid_${point.pid}`;
    const existing = dataRef.current[key] || [];
    const updated = [...existing, point].slice(-MAX_DATA_POINTS);
    dataRef.current = { ...dataRef.current, [key]: updated };

    setData({ ...dataRef.current });
    setLatestValues((prev) => ({ ...prev, [point.pid]: point }));
  }, [ecuId]);

  useEffect(() => {
    wsClient.setOnLiveData(handleLiveData);
    wsClient.setOnConnectionChange(setConnected);
    wsClient.connect();
    wsClient.subscribe(ecuId, pids);

    return () => {
      wsClient.unsubscribe(ecuId);
    };
  }, [ecuId, pids, handleLiveData]);

  return { data, connected, latestValues };
}
