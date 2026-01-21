import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import type { Project, Service } from '../types/events';

interface WelcomeScreenProps {
  onCreateProject: (project: Project) => Promise<void>;
}

function generateId(): string {
  return Math.random().toString(36).substring(2, 9);
}

export function WelcomeScreen({ onCreateProject }: WelcomeScreenProps) {
  const [projectName, setProjectName] = useState('');
  const [services, setServices] = useState<Service[]>([
    { id: generateId(), name: '', path: '', command: '', detect_url: false },
  ]);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleAddService = () => {
    setServices([
      ...services,
      { id: generateId(), name: '', path: '', command: '', detect_url: false },
    ]);
  };

  const handleRemoveService = (index: number) => {
    if (services.length > 1) {
      setServices(services.filter((_, i) => i !== index));
    }
  };

  const handleServiceChange = (
    index: number,
    field: keyof Service,
    value: string | boolean
  ) => {
    setServices(
      services.map((s, i) => (i === index ? { ...s, [field]: value } : s))
    );
  };

  const handleSelectPath = async (index: number) => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select project directory',
      });
      if (selected) {
        handleServiceChange(index, 'path', selected as string);
      }
    } catch (err) {
      console.error('Failed to open dialog:', err);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!projectName.trim()) {
      setError('Project name is required');
      return;
    }

    const validServices = services.filter(
      (s) => s.name.trim() && s.path.trim() && s.command.trim()
    );

    if (validServices.length === 0) {
      setError('At least one service with name, path, and command is required');
      return;
    }

    setIsSubmitting(true);
    try {
      const project: Project = {
        id: generateId(),
        name: projectName.trim(),
        services: validServices.map((s) => ({
          ...s,
          name: s.name.trim(),
          path: s.path.trim(),
          command: s.command.trim(),
        })),
      };
      await onCreateProject(project);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="welcome-screen">
      <div className="welcome-card">
        <h1>Dev Stack Launcher</h1>
        <p className="welcome-subtitle">Create your first project to get started</p>

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="project-name">Project Name</label>
            <input
              id="project-name"
              type="text"
              value={projectName}
              onChange={(e) => setProjectName(e.target.value)}
              placeholder="My Project"
              autoFocus
            />
          </div>

          <div className="services-section">
            <div className="services-header">
              <h3>Services</h3>
              <button
                type="button"
                className="btn btn-secondary btn-small"
                onClick={handleAddService}
              >
                + Add Service
              </button>
            </div>

            {services.map((service, index) => (
              <div key={service.id} className="service-form">
                <div className="service-form-header">
                  <span className="service-number">Service {index + 1}</span>
                  {services.length > 1 && (
                    <button
                      type="button"
                      className="btn-icon btn-danger"
                      onClick={() => handleRemoveService(index)}
                      title="Remove service"
                    >
                      x
                    </button>
                  )}
                </div>

                <div className="form-row">
                  <div className="form-group">
                    <label>Name</label>
                    <input
                      type="text"
                      value={service.name}
                      onChange={(e) =>
                        handleServiceChange(index, 'name', e.target.value)
                      }
                      placeholder="Frontend"
                    />
                  </div>
                  <div className="form-group">
                    <label>Command</label>
                    <input
                      type="text"
                      value={service.command}
                      onChange={(e) =>
                        handleServiceChange(index, 'command', e.target.value)
                      }
                      placeholder="pnpm dev"
                    />
                  </div>
                </div>

                <div className="form-group">
                  <label>Path</label>
                  <div className="path-input">
                    <input
                      type="text"
                      value={service.path}
                      onChange={(e) =>
                        handleServiceChange(index, 'path', e.target.value)
                      }
                      placeholder="/path/to/project"
                    />
                    <button
                      type="button"
                      className="btn btn-secondary"
                      onClick={() => handleSelectPath(index)}
                    >
                      Browse
                    </button>
                  </div>
                </div>

                <div className="form-group checkbox-group">
                  <label>
                    <input
                      type="checkbox"
                      checked={service.detect_url}
                      onChange={(e) =>
                        handleServiceChange(index, 'detect_url', e.target.checked)
                      }
                    />
                    Detect URL (for dev servers like Vite)
                  </label>
                </div>
              </div>
            ))}
          </div>

          {error && <div className="error-message">{error}</div>}

          <button
            type="submit"
            className="btn btn-primary btn-large"
            disabled={isSubmitting}
          >
            {isSubmitting ? 'Creating...' : 'Create Project'}
          </button>
        </form>
      </div>
    </div>
  );
}
