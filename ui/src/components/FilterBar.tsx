import type { Filters, LogSource, LogLevel } from '../types/events';

interface FilterBarProps {
  filters: Filters;
  onChange: (filters: Filters) => void;
}

export function FilterBar({ filters, onChange }: FilterBarProps) {
  const sources: { value: LogSource; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'frontend', label: 'Frontend' },
    { value: 'backend', label: 'Backend' },
    { value: 'system', label: 'System' },
  ];

  const levels: { value: LogLevel; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'normal', label: 'Normal' },
    { value: 'error', label: 'Error' },
  ];

  return (
    <div className="filter-bar">
      <div className="filter-group">
        <span className="filter-label">Source:</span>
        {sources.map(({ value, label }) => (
          <button
            key={value}
            className={`filter-btn ${filters.source === value ? 'active' : ''}`}
            onClick={() => onChange({ ...filters, source: value })}
          >
            {label}
          </button>
        ))}
      </div>

      <div className="filter-group">
        <span className="filter-label">Level:</span>
        {levels.map(({ value, label }) => (
          <button
            key={value}
            className={`filter-btn ${filters.level === value ? 'active' : ''}`}
            onClick={() => onChange({ ...filters, level: value })}
          >
            {label}
          </button>
        ))}
      </div>
    </div>
  );
}
