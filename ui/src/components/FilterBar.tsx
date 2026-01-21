import type { Filters, LogLevel } from '../types/events';

interface FilterBarProps {
  filters: Filters;
  onFiltersChange: (filters: Filters) => void;
  sourceOptions: Array<{ value: string; label: string }>;
}

export function FilterBar({ filters, onFiltersChange, sourceOptions }: FilterBarProps) {
  const levels: { value: LogLevel; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'normal', label: 'Normal' },
    { value: 'error', label: 'Error' },
  ];

  return (
    <div className="filter-bar">
      <div className="filter-group">
        <span className="filter-label">Source:</span>
        {sourceOptions.map(({ value, label }) => (
          <button
            key={value}
            className={`filter-btn ${filters.source === value ? 'active' : ''}`}
            onClick={() => onFiltersChange({ ...filters, source: value })}
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
            onClick={() => onFiltersChange({ ...filters, level: value })}
          >
            {label}
          </button>
        ))}
      </div>
    </div>
  );
}
