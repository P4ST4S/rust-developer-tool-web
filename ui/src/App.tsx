import { useRef, useState } from 'react';
import { Terminal, TerminalHandle } from './components/Terminal';
import { Controls } from './components/Controls';
import { FilterBar } from './components/FilterBar';
import { SearchBar } from './components/SearchBar';
import { useProcessControl } from './hooks/useProcessControl';
import { useLogStream } from './hooks/useLogStream';
import type { Filters } from './types/events';
import './styles/app.css';

function App() {
  const terminalRef = useRef<TerminalHandle>(null);
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');
  const [filters, setFilters] = useState<Filters>({
    source: 'all',
    level: 'all',
  });

  const {
    status,
    loading,
    startFrontend,
    stopFrontend,
    startBackend,
    stopBackend,
    openBrowser,
  } = useProcessControl();

  const { clearLogs } = useLogStream(terminalRef, filters);

  const toggleTheme = () => {
    const newTheme = theme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    terminalRef.current?.setTheme(newTheme);
  };

  return (
    <div className={`app ${theme}`}>
      <header className="header">
        <h1>Dev Stack Launcher</h1>
        <button onClick={toggleTheme} className="theme-toggle">
          {theme === 'dark' ? 'Light' : 'Dark'}
        </button>
      </header>

      <Controls
        status={status}
        loading={loading}
        onStartFrontend={startFrontend}
        onStopFrontend={stopFrontend}
        onStartBackend={startBackend}
        onStopBackend={stopBackend}
        onOpenBrowser={openBrowser}
        onClearLogs={clearLogs}
      />

      <FilterBar filters={filters} onChange={setFilters} />

      <SearchBar terminalRef={terminalRef} />

      <div className="terminal-container">
        <Terminal ref={terminalRef} theme={theme} />
      </div>
    </div>
  );
}

export default App;
