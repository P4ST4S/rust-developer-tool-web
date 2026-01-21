import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useEffect, useRef, useCallback, useState } from 'react';
import type { LogEvent, Filters } from '../types/events';
import type { TerminalHandle } from '../components/Terminal';

const MAX_LOGS = 20000;

export function useLogStream(
  terminalRef: React.RefObject<TerminalHandle | null>,
  filters: Filters,
  projectId: string
) {
  const [logs, setLogs] = useState<LogEvent[]>([]);
  const logsRef = useRef<LogEvent[]>([]);
  const filtersRef = useRef(filters);
  const projectIdRef = useRef(projectId);
  filtersRef.current = filters;
  projectIdRef.current = projectId;

  const shouldDisplayLog = useCallback((log: LogEvent): boolean => {
    const currentFilters = filtersRef.current;
    const currentProjectId = projectIdRef.current;

    // Filter by project
    if (log.project_id !== currentProjectId) {
      return false;
    }

    // Filter by source
    if (currentFilters.source !== 'all' && currentFilters.source !== log.source) {
      return false;
    }

    // Filter by level
    if (currentFilters.level !== 'all' && currentFilters.level !== log.level) {
      return false;
    }

    return true;
  }, []);

  const rewriteTerminal = useCallback(() => {
    if (!terminalRef.current) return;

    terminalRef.current.clear();
    const filteredLogs = logsRef.current.filter(shouldDisplayLog);
    for (const log of filteredLogs) {
      terminalRef.current.writeln(log.text);
    }
  }, [terminalRef, shouldDisplayLog]);

  useEffect(() => {
    rewriteTerminal();
  }, [filters.source, filters.level, rewriteTerminal]);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;
    let mounted = true;

    listen<LogEvent>('log', (event) => {
      if (!mounted) return;

      const newLog = event.payload;

      // Only store logs for this project
      if (newLog.project_id !== projectIdRef.current) {
        return;
      }

      logsRef.current = [...logsRef.current, newLog];
      if (logsRef.current.length > MAX_LOGS) {
        logsRef.current = logsRef.current.slice(-MAX_LOGS);
      }

      setLogs([...logsRef.current]);

      if (shouldDisplayLog(newLog) && terminalRef.current) {
        terminalRef.current.writeln(newLog.text);
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      mounted = false;
      unlisten?.();
    };
  }, [terminalRef, shouldDisplayLog]);

  // Clear logs when project changes
  useEffect(() => {
    logsRef.current = [];
    setLogs([]);
    terminalRef.current?.clear();
  }, [projectId, terminalRef]);

  const clearLogs = useCallback(() => {
    logsRef.current = [];
    setLogs([]);
    terminalRef.current?.clear();
  }, [terminalRef]);

  return { logs, clearLogs };
}
