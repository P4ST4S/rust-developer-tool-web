import { useState } from 'react';
import { WelcomeScreen } from './components/WelcomeScreen';
import { ProjectTabs } from './components/ProjectTabs';
import { ProjectView } from './components/ProjectView';
import { ConfigModal } from './components/ConfigModal';
import { useConfig } from './hooks/useConfig';
import './styles/app.css';

function App() {
  const {
    config,
    loading,
    addProject,
    updateProject,
    deleteProject,
    addService,
    updateService,
    deleteService,
  } = useConfig();

  const [theme, setTheme] = useState<'dark' | 'light'>('dark');
  const [activeProjectId, setActiveProjectId] = useState<string | null>(null);
  const [showConfigModal, setShowConfigModal] = useState(false);

  const toggleTheme = () => {
    setTheme((prev) => (prev === 'dark' ? 'light' : 'dark'));
  };

  if (loading) {
    return (
      <div className={`app ${theme} loading-screen`}>
        <div className="loading-spinner">Loading...</div>
      </div>
    );
  }

  if (!config || config.projects.length === 0) {
    return (
      <div className={`app ${theme}`}>
        <WelcomeScreen onCreateProject={addProject} />
      </div>
    );
  }

  const showTabs = config.projects.length > 1;
  const currentProjectId = activeProjectId || config.projects[0].id;
  const currentProject = config.projects.find((p) => p.id === currentProjectId);

  if (!currentProject) {
    return (
      <div className={`app ${theme}`}>
        <WelcomeScreen onCreateProject={addProject} />
      </div>
    );
  }

  return (
    <div className={`app ${theme}`}>
      {showTabs && (
        <ProjectTabs
          projects={config.projects}
          activeProjectId={currentProjectId}
          onTabChange={setActiveProjectId}
          onManageProjects={() => setShowConfigModal(true)}
        />
      )}
      {!showTabs && (
        <div className="single-project-header">
          <h1>{currentProject.name}</h1>
          <button
            className="btn btn-secondary btn-small"
            onClick={() => setShowConfigModal(true)}
          >
            Settings
          </button>
        </div>
      )}
      <ProjectView
        key={currentProject.id}
        project={currentProject}
        theme={theme}
        onThemeToggle={toggleTheme}
      />
      {showConfigModal && (
        <ConfigModal
          config={config}
          onClose={() => setShowConfigModal(false)}
          onAddProject={addProject}
          onUpdateProject={updateProject}
          onDeleteProject={deleteProject}
          onAddService={addService}
          onUpdateService={updateService}
          onDeleteService={deleteService}
        />
      )}
    </div>
  );
}

export default App;
