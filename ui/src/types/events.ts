export interface LogEvent {
  source: 'frontend' | 'backend' | 'system';
  level: 'normal' | 'error';
  text: string;
  timestamp: string;
}

export interface StatusEvent {
  frontend_running: boolean;
  backend_running: boolean;
  frontend_url: string | null;
}

export type LogSource = 'frontend' | 'backend' | 'system' | 'all';
export type LogLevel = 'normal' | 'error' | 'all';

export interface Filters {
  source: LogSource;
  level: LogLevel;
}
