import axios from 'axios';
import type {
  CreateSessionRequest,
  SessionResponse,
  DtcResponse,
  ClearDtcRequest,
  ClearDtcResponse,
  EcuInfo,
  HealthResponse,
  PaginatedResponse,
} from '../types';

// In production, this would be configured per environment
const API_BASE_URL = 'http://localhost:8080/api/v1';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
  timeout: 10000,
});

export const healthCheck = async (): Promise<HealthResponse> => {
  const { data } = await api.get('/health');
  return data;
};

export const createSession = async (
  req: CreateSessionRequest
): Promise<SessionResponse> => {
  const { data } = await api.post('/diagnostics/session', req);
  return data;
};

export const readDtcs = async (
  ecuId?: number,
  statusMask?: number
): Promise<DtcResponse[]> => {
  const params: Record<string, string> = {};
  if (ecuId !== undefined) params.ecu_id = ecuId.toString();
  if (statusMask !== undefined) params.status_mask = statusMask.toString();
  const { data } = await api.get('/diagnostics/dtc', { params });
  return data;
};

export const clearDtcs = async (
  req: ClearDtcRequest
): Promise<ClearDtcResponse> => {
  const { data } = await api.post('/diagnostics/clear-dtc', req);
  return data;
};

export const listEcus = async (
  manufacturer?: string,
  page?: number,
  perPage?: number
): Promise<PaginatedResponse<EcuInfo>> => {
  const params: Record<string, string> = {};
  if (manufacturer) params.manufacturer = manufacturer;
  if (page) params.page = page.toString();
  if (perPage) params.per_page = perPage.toString();
  const { data } = await api.get('/data/ecus', { params });
  return data;
};

export default api;
