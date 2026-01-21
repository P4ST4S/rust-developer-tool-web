import type { Project } from '../types/events';

interface ProjectTabsProps {
  projects: Project[];
  activeProjectId: string;
  onTabChange: (projectId: string) => void;
  onManageProjects: () => void;
}

export function ProjectTabs({
  projects,
  activeProjectId,
  onTabChange,
  onManageProjects,
}: ProjectTabsProps) {
  return (
    <div className="project-tabs">
      <div className="tabs-list">
        {projects.map((project) => (
          <button
            key={project.id}
            className={`tab ${project.id === activeProjectId ? 'active' : ''}`}
            onClick={() => onTabChange(project.id)}
          >
            {project.name}
          </button>
        ))}
      </div>
      <button
        className="btn btn-secondary btn-small manage-btn"
        onClick={onManageProjects}
        title="Manage Projects"
      >
        +
      </button>
    </div>
  );
}
