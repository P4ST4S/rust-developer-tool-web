import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import type { Config, Project, Service } from '../types/events';

interface ConfigModalProps {
  config: Config;
  onClose: () => void;
  onAddProject: (project: Project) => Promise<void>;
  onUpdateProject: (projectId: string, updates: Partial<Project>) => Promise<void>;
  onDeleteProject: (projectId: string) => Promise<void>;
}

function generateId(): string {
  return Math.random().toString(36).substring(2, 9);
}

type ModalView = 'list' | 'add-project' | 'edit-project';

export function ConfigModal({
  config,
  onClose,
  onAddProject,
  onUpdateProject,
  onDeleteProject,
}: ConfigModalProps) {
  const [view, setView] = useState<ModalView>('list');
  const [editingProject, setEditingProject] = useState<Project | null>(null);
  const [newProjectName, setNewProjectName] = useState('');
  const [newServices, setNewServices] = useState<Service[]>([
    { id: generateId(), name: '', path: '', command: '', detect_url: false },
  ]);
  const [error, setError] = useState<string | null>(null);

  const handleSelectPath = async (
    index: number,
    services: Service[],
    setServices: (s: Service[]) => void
  ) => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select project directory',
      });
      if (selected) {
        setServices(
          services.map((s, i) =>
            i === index ? { ...s, path: selected as string } : s
          )
        );
      }
    } catch (err) {
      console.error('Failed to open dialog:', err);
    }
  };

  const handleAddNewProject = async () => {
    setError(null);
    if (!newProjectName.trim()) {
      setError('Project name is required');
      return;
    }
    const validServices = newServices.filter(
      (s) => s.name.trim() && s.path.trim() && s.command.trim()
    );
    if (validServices.length === 0) {
      setError('At least one complete service is required');
      return;
    }

    try {
      await onAddProject({
        id: generateId(),
        name: newProjectName.trim(),
        services: validServices,
      });
      setView('list');
      setNewProjectName('');
      setNewServices([
        { id: generateId(), name: '', path: '', command: '', detect_url: false },
      ]);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleDeleteProject = async (projectId: string) => {
    if (confirm('Are you sure you want to delete this project?')) {
      await onDeleteProject(projectId);
    }
  };

  const handleEditProject = (project: Project) => {
    setEditingProject({ ...project, services: [...project.services] });
    setView('edit-project');
  };

  const handleSaveProject = async () => {
    if (!editingProject) return;
    setError(null);

    if (!editingProject.name.trim()) {
      setError('Project name is required');
      return;
    }

    const validServices = editingProject.services.filter(
      (s) => s.name.trim() && s.path.trim() && s.command.trim()
    );
    if (validServices.length === 0) {
      setError('At least one complete service is required');
      return;
    }

    try {
      await onUpdateProject(editingProject.id, {
        name: editingProject.name.trim(),
        services: validServices,
      });
      setView('list');
      setEditingProject(null);
    } catch (err) {
      setError(String(err));
    }
  };

  const renderProjectList = () => (
    <>
      <div className="modal-header">
        <h2>Manage Projects</h2>
        <button className="btn-close" onClick={onClose}>
          x
        </button>
      </div>
      <div className="modal-body">
        <div className="project-list">
          {config.projects.map((project) => (
            <div key={project.id} className="project-item">
              <div className="project-info">
                <strong>{project.name}</strong>
                <span className="service-count">
                  {project.services.length} service(s)
                </span>
              </div>
              <div className="project-actions">
                <button
                  className="btn btn-secondary btn-small"
                  onClick={() => handleEditProject(project)}
                >
                  Edit
                </button>
                <button
                  className="btn btn-danger btn-small"
                  onClick={() => handleDeleteProject(project.id)}
                >
                  Delete
                </button>
              </div>
            </div>
          ))}
        </div>
        <button
          className="btn btn-primary"
          onClick={() => setView('add-project')}
        >
          + Add Project
        </button>
      </div>
    </>
  );

  const renderServiceForm = (
    services: Service[],
    setServices: (s: Service[]) => void
  ) => (
    <div className="services-section">
      <div className="services-header">
        <h3>Services</h3>
        <button
          type="button"
          className="btn btn-secondary btn-small"
          onClick={() =>
            setServices([
              ...services,
              { id: generateId(), name: '', path: '', command: '', detect_url: false },
            ])
          }
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
                onClick={() => setServices(services.filter((_, i) => i !== index))}
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
                  setServices(
                    services.map((s, i) =>
                      i === index ? { ...s, name: e.target.value } : s
                    )
                  )
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
                  setServices(
                    services.map((s, i) =>
                      i === index ? { ...s, command: e.target.value } : s
                    )
                  )
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
                  setServices(
                    services.map((s, i) =>
                      i === index ? { ...s, path: e.target.value } : s
                    )
                  )
                }
                placeholder="/path/to/project"
              />
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() =>
                  handleSelectPath(index, services, setServices)
                }
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
                  setServices(
                    services.map((s, i) =>
                      i === index ? { ...s, detect_url: e.target.checked } : s
                    )
                  )
                }
              />
              Detect URL (for dev servers like Vite)
            </label>
          </div>
        </div>
      ))}
    </div>
  );

  const renderAddProject = () => (
    <>
      <div className="modal-header">
        <button className="btn-back" onClick={() => setView('list')}>
          &lt; Back
        </button>
        <h2>Add Project</h2>
        <button className="btn-close" onClick={onClose}>
          x
        </button>
      </div>
      <div className="modal-body">
        <div className="form-group">
          <label>Project Name</label>
          <input
            type="text"
            value={newProjectName}
            onChange={(e) => setNewProjectName(e.target.value)}
            placeholder="My Project"
            autoFocus
          />
        </div>

        {renderServiceForm(newServices, setNewServices)}

        {error && <div className="error-message">{error}</div>}

        <button className="btn btn-primary" onClick={handleAddNewProject}>
          Create Project
        </button>
      </div>
    </>
  );

  const renderEditProject = () => {
    if (!editingProject) return null;

    return (
      <>
        <div className="modal-header">
          <button className="btn-back" onClick={() => setView('list')}>
            &lt; Back
          </button>
          <h2>Edit Project</h2>
          <button className="btn-close" onClick={onClose}>
            x
          </button>
        </div>
        <div className="modal-body">
          <div className="form-group">
            <label>Project Name</label>
            <input
              type="text"
              value={editingProject.name}
              onChange={(e) =>
                setEditingProject({ ...editingProject, name: e.target.value })
              }
              placeholder="My Project"
            />
          </div>

          {renderServiceForm(
            editingProject.services,
            (services) => setEditingProject({ ...editingProject, services })
          )}

          {error && <div className="error-message">{error}</div>}

          <button className="btn btn-primary" onClick={handleSaveProject}>
            Save Changes
          </button>
        </div>
      </>
    );
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        {view === 'list' && renderProjectList()}
        {view === 'add-project' && renderAddProject()}
        {view === 'edit-project' && renderEditProject()}
      </div>
    </div>
  );
}
