import { useRef, useState } from 'react';
import type { Project, Filters } from '../types/events';
import { Terminal, TerminalHandle } from './Terminal';
import { ServiceControls } from './ServiceControls';
import { FilterBar } from './FilterBar';
import { SearchBar } from './SearchBar';
import { useLogStream } from '../hooks/useLogStream';
import { useProcessControl } from '../hooks/useProcessControl';

interface ProjectViewProps {
  project: Project;
  theme: 'dark' | 'light';
  onThemeToggle: () => void;
}

export function ProjectView({ project, theme, onThemeToggle }: ProjectViewProps) {
  const terminalRef = useRef<TerminalHandle>(null);
  const [filters, setFilters] = useState<Filters>({
    source: 'all',
    level: 'all',
  });

  const { logs: _logs, clearLogs } = useLogStream(terminalRef, filters, project.id);
  const {
    startService,
    stopService,
    openBrowser,
    getServiceStatus,
    isServiceLoading,
  } = useProcessControl(project.id);

  const handleSearch = (query: string) => {
    terminalRef.current?.search(query);
  };

  const handleSearchNext = () => {
    terminalRef.current?.findNext();
  };

  const handleSearchPrev = () => {
    terminalRef.current?.findPrevious();
  };

  // Build source options dynamically from project services
  const sourceOptions: Array<{ value: string; label: string }> = [
    { value: 'all', label: 'All' },
    { value: 'system', label: 'System' },
    ...project.services.map((s) => ({
      value: s.name.toLowerCase(),
      label: s.name,
    })),
  ];

  return (
    <div className="project-view">
      <header className="project-header">
        <ServiceControls
          services={project.services}
          getServiceStatus={getServiceStatus}
          isServiceLoading={isServiceLoading}
          onStart={startService}
          onStop={stopService}
          onOpenBrowser={openBrowser}
        />
        <div className="header-controls">
          <FilterBar
            filters={filters}
            onFiltersChange={setFilters}
            sourceOptions={sourceOptions}
          />
          <SearchBar
            onSearch={handleSearch}
            onNext={handleSearchNext}
            onPrev={handleSearchPrev}
          />
          <button
            className="btn btn-secondary btn-icon"
            onClick={clearLogs}
            title="Clear logs"
          >
            Clear
          </button>
          <button
            className="btn btn-secondary btn-icon"
            onClick={onThemeToggle}
            title={`Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`}
          >
            {theme === 'dark' ? '☀' : '☾'}
          </button>
        </div>
      </header>
      <main className="terminal-container">
        <Terminal ref={terminalRef} theme={theme} />
      </main>
    </div>
  );
}
