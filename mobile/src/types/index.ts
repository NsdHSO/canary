// Shared types - identical to web/src/types/index.ts for code sharing

export interface HealthResponse {
  status: string;
  version: string;
  uptime_secs: number;
}

export interface CreateSessionRequest {
  ecu_id: number;
  session_type?: string;
  adapter?: string;
}

export interface SessionResponse {
  session_id: string;
  ecu_id: number;
  session_type: string;
  status: string;
  created_at: string;
}

export interface DtcResponse {
  code: string;
  description: string;
  status: number;
  severity: string;
  ecu_id: number;
  system: string;
}

export interface ClearDtcRequest {
  ecu_id: number;
  codes?: string[];
}

export interface ClearDtcResponse {
  success: boolean;
  cleared_count: number;
  message: string;
}

export interface EcuInfo {
  id: string;
  manufacturer: string;
  model: string;
  year_range: string;
  ecu_type: string;
  can_id: number;
  protocols: string[];
}

export interface LiveDataPoint {
  timestamp_ms: number;
  ecu_id: number;
  pid: number;
  name: string;
  value: number;
  unit: string;
  min?: number;
  max?: number;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
}

export type WsMessage =
  | { type: 'Subscribe'; data: { ecu_id: number; pids: number[] } }
  | { type: 'Unsubscribe'; data: { ecu_id: number } }
  | { type: 'LiveData'; data: LiveDataPoint }
  | { type: 'CanFrame'; data: { id: number; data: number[]; timestamp_us: number } }
  | { type: 'Error'; data: { message: string } }
  | { type: 'Ping' }
  | { type: 'Pong' };
