// Config types
export interface Service {
  id: string;
  name: string;
  path: string;
  command: string;
  detect_url: boolean;
}

export interface Project {
  id: string;
  name: string;
  services: Service[];
}

export interface Config {
  version: number;
  active_project: string | null;
  projects: Project[];
}

// Log types
export interface LogEvent {
  source: string;
  level: 'normal' | 'error';
  text: string;
  timestamp: string;
  project_id: string;
}

// Status types
export interface ServiceStatus {
  running: boolean;
  url: string | null;
}

export interface StatusEvent {
  services: Record<string, ServiceStatus>;
}

// Filter types
export type LogLevel = 'normal' | 'error' | 'all';

export interface Filters {
  source: 'all' | 'system' | string; // string = service name
  level: LogLevel;
}
