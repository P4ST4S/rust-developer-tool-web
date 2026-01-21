import type { Service, ServiceStatus } from '../types/events';

interface ServiceControlsProps {
  services: Service[];
  getServiceStatus: (serviceId: string) => ServiceStatus;
  isServiceLoading: (serviceId: string) => boolean;
  onStart: (serviceId: string) => void;
  onStop: (serviceId: string) => void;
  onOpenBrowser: (url: string) => void;
}

export function ServiceControls({
  services,
  getServiceStatus,
  isServiceLoading,
  onStart,
  onStop,
  onOpenBrowser,
}: ServiceControlsProps) {
  return (
    <div className="service-controls">
      {services.map((service) => {
        const status = getServiceStatus(service.id);
        const loading = isServiceLoading(service.id);

        return (
          <div key={service.id} className="service-control">
            <span
              className={`status-indicator ${status.running ? 'running' : 'stopped'}`}
            />
            <span className="service-name">{service.name}</span>
            {status.running ? (
              <button
                className="btn btn-danger btn-small"
                onClick={() => onStop(service.id)}
                disabled={loading}
              >
                {loading ? '...' : 'Stop'}
              </button>
            ) : (
              <button
                className="btn btn-primary btn-small"
                onClick={() => onStart(service.id)}
                disabled={loading}
              >
                {loading ? '...' : 'Start'}
              </button>
            )}
            {status.url && (
              <button
                className="btn btn-secondary btn-small"
                onClick={() => onOpenBrowser(status.url!)}
                title={status.url}
              >
                Open
              </button>
            )}
          </div>
        );
      })}
    </div>
  );
}
