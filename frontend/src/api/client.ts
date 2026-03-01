/**
 * Kroma Backend API Client
 * Base URL configurable via environment variable
 */

const BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:8788';

export interface ApiResponse<T> {
  ok: boolean;
  error?: string;
  error_kind?: 'validation' | 'provider' | 'infra' | 'policy' | 'unknown';
  error_code?: string;
  data?: T;
}

export interface Project {
  id: string;
  slug: string;
  name: string;
  description: string;
  status: 'active' | 'archived' | 'deleted';
  created_at: string;
  updated_at: string;
}

export interface CreateProjectRequest {
  name: string;
  slug?: string;
  description?: string;
}

class ApiClient {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  setToken(token: string | null) {
    this.token = token;
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown
  ): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
    };

    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }

    const options: RequestInit = {
      method,
      headers,
    };

    if (body && method !== 'GET') {
      options.body = JSON.stringify(body);
    }

    const response = await fetch(url, options);
    const data = await response.json();

    if (!response.ok) {
      throw new ApiError(data);
    }

    return data as T;
  }

  // Auth endpoints (J00)
  async bootstrapToken(): Promise<ApiResponse<{ token: string }>> {
    return this.request('POST', '/api/auth/token', { note: 'Kroma Frontend' });
  }

  // Projects endpoints (J01)
  async listProjects(): Promise<ApiResponse<{ projects: Project[]; count: number }>> {
    return this.request('GET', '/api/projects');
  }

  async createProject(data: CreateProjectRequest): Promise<ApiResponse<{ project: Project }>> {
    return this.request('POST', '/api/projects', data);
  }

  async getProject(slug: string): Promise<ApiResponse<{ project: Project }>> {
    return this.request('GET', `/api/projects/${slug}`);
  }

  // Health check
  async health(): Promise<ApiResponse<{ status: string; service: string }>> {
    return this.request('GET', '/health');
  }
}

export class ApiError extends Error {
  data: ApiResponse<unknown>;

  constructor(data: ApiResponse<unknown>) {
    super(data.error || 'API request failed');
    this.data = data;
    this.name = 'ApiError';
  }

  get errorKind() {
    return this.data.error_kind;
  }

  get errorCode() {
    return this.data.error_code;
  }
}

export const apiClient = new ApiClient(BASE_URL);
export default apiClient;
