import type { StatusEvent } from '../types/events';

interface ControlsProps {
  status: StatusEvent;
  loading: { frontend: boolean; backend: boolean };
  onStartFrontend: () => void;
  onStopFrontend: () => void;
  onStartBackend: () => void;
  onStopBackend: () => void;
  onOpenBrowser: () => void;
  onClearLogs: () => void;
}

export function Controls({
  status,
  loading,
  onStartFrontend,
  onStopFrontend,
  onStartBackend,
  onStopBackend,
  onOpenBrowser,
  onClearLogs,
}: ControlsProps) {
  const canOpenBrowser =
    status.frontend_running &&
    status.backend_running &&
    status.frontend_url !== null;

  return (
    <div className="controls">
      <div className="control-row">
        <span className="label">Frontend:</span>
        <button
          onClick={onStartFrontend}
          disabled={status.frontend_running || loading.frontend}
          className="btn btn-start"
        >
          Start
        </button>
        <button
          onClick={onStopFrontend}
          disabled={!status.frontend_running || loading.frontend}
          className="btn btn-stop"
        >
          Stop
        </button>
        <span
          className={`status ${status.frontend_running ? 'running' : 'stopped'}`}
        >
          {status.frontend_running ? 'Running' : 'Stopped'}
        </span>
      </div>

      <div className="control-row">
        <span className="label">Backend:</span>
        <button
          onClick={onStartBackend}
          disabled={status.backend_running || loading.backend}
          className="btn btn-start"
        >
          Start
        </button>
        <button
          onClick={onStopBackend}
          disabled={!status.backend_running || loading.backend}
          className="btn btn-stop"
        >
          Stop
        </button>
        <span
          className={`status ${status.backend_running ? 'running' : 'stopped'}`}
        >
          {status.backend_running ? 'Running' : 'Stopped'}
        </span>
      </div>

      <div className="control-row actions">
        <button
          onClick={onOpenBrowser}
          disabled={!canOpenBrowser}
          className="btn btn-primary"
        >
          Open Frontend in Browser
        </button>
        <button onClick={onClearLogs} className="btn">
          Clear Logs
        </button>
      </div>
    </div>
  );
}
