import { useState, useCallback, useEffect, useRef } from 'react';

interface SearchBarProps {
  onSearch: (query: string) => void;
  onNext: () => void;
  onPrev: () => void;
}

export function SearchBar({ onSearch, onNext, onPrev }: SearchBarProps) {
  const [query, setQuery] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  const handleSearch = useCallback(
    (value: string) => {
      setQuery(value);
      if (value) {
        onSearch(value);
      }
    },
    [onSearch]
  );

  const handleClear = useCallback(() => {
    setQuery('');
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
        e.preventDefault();
        inputRef.current?.focus();
      }
      if (e.key === 'Enter' && document.activeElement === inputRef.current) {
        e.preventDefault();
        if (e.shiftKey) {
          onPrev();
        } else {
          onNext();
        }
      }
      if (e.key === 'Escape' && document.activeElement === inputRef.current) {
        handleClear();
        inputRef.current?.blur();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onNext, onPrev, handleClear]);

  return (
    <div className="search-bar">
      <input
        ref={inputRef}
        type="text"
        placeholder="Search... (Cmd+F)"
        value={query}
        onChange={(e) => handleSearch(e.target.value)}
        className="search-input"
      />
      <button
        onClick={onPrev}
        disabled={!query}
        title="Previous (Shift+Enter)"
        className="search-btn"
      >
        ^
      </button>
      <button
        onClick={onNext}
        disabled={!query}
        title="Next (Enter)"
        className="search-btn"
      >
        v
      </button>
      <button
        onClick={handleClear}
        disabled={!query}
        title="Clear (Esc)"
        className="search-btn"
      >
        x
      </button>
    </div>
  );
}
