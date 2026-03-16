import type { Config, OutputTransform, StatusResponse } from './types';

const BASE = '';

async function request<T>(url: string, init?: RequestInit): Promise<T> {
  const resp = await fetch(BASE + url, { cache: 'no-store', ...init });
  if (!resp.ok) {
    const body = await resp.json().catch(() => ({ error: resp.statusText }));
    throw new Error(body.error || resp.statusText);
  }
  return resp.json();
}

export async function getStatus(): Promise<StatusResponse> {
  return request('/api/status');
}

export async function getConfig(): Promise<Config> {
  return request('/api/config');
}

export async function putConfig(config: Config): Promise<void> {
  await request('/api/config', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(config),
  });
}

export async function updateOutput(id: string, transform: OutputTransform): Promise<void> {
  await request(`/api/config/output/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(transform),
  });
}

export async function saveConfig(): Promise<void> {
  await request('/api/config/save', { method: 'POST' });
}

export async function loadConfig(path: string): Promise<void> {
  await request('/api/config/load', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path }),
  });
}

export async function startPipeline(): Promise<void> {
  await request('/api/pipeline/start', { method: 'POST' });
}

export async function stopPipeline(): Promise<void> {
  await request('/api/pipeline/stop', { method: 'POST' });
}
