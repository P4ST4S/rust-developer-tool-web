import { useState, useCallback, useEffect, useRef } from 'react';
import type { TerminalHandle } from './Terminal';

interface SearchBarProps {
  terminalRef: React.RefObject<TerminalHandle | null>;
}

export function SearchBar({ terminalRef }: SearchBarProps) {
  const [query, setQuery] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  const handleSearch = useCallback(
    (value: string) => {
      setQuery(value);
      if (value && terminalRef.current) {
        terminalRef.current.search(value);
      }
    },
    [terminalRef]
  );

  const handleFindNext = useCallback(() => {
    terminalRef.current?.findNext();
  }, [terminalRef]);

  const handleFindPrevious = useCallback(() => {
    terminalRef.current?.findPrevious();
  }, [terminalRef]);

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
          handleFindPrevious();
        } else {
          handleFindNext();
        }
      }
      if (e.key === 'Escape' && document.activeElement === inputRef.current) {
        handleClear();
        inputRef.current?.blur();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleFindNext, handleFindPrevious, handleClear]);

  return (
    <div className="search-bar">
      <input
        ref={inputRef}
        type="text"
        placeholder="Search logs... (Cmd+F)"
        value={query}
        onChange={(e) => handleSearch(e.target.value)}
        className="search-input"
      />
      <button
        onClick={handleFindPrevious}
        disabled={!query}
        title="Previous (Shift+Enter)"
        className="search-btn"
      >
        ^
      </button>
      <button
        onClick={handleFindNext}
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
