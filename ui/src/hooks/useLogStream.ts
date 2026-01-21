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
    let unlistenLog: UnlistenFn | null = null;
    let unlistenBatch: UnlistenFn | null = null;
    let mounted = true;

    const appendLogs = (incomingLogs: LogEvent[]) => {
      if (!incomingLogs.length) return;

      let didAppend = false;
      let updatedLogs = logsRef.current;

      for (const newLog of incomingLogs) {
        if (newLog.project_id !== projectIdRef.current) {
          continue;
        }

        if (!didAppend) {
          updatedLogs = [...updatedLogs];
          didAppend = true;
        }

        updatedLogs.push(newLog);
        if (shouldDisplayLog(newLog) && terminalRef.current) {
          terminalRef.current.writeln(newLog.text);
        }
      }

      if (!didAppend) return;

      if (updatedLogs.length > MAX_LOGS) {
        updatedLogs = updatedLogs.slice(-MAX_LOGS);
      }

      logsRef.current = updatedLogs;
      setLogs([...updatedLogs]);
    };

    listen<LogEvent>('log', (event) => {
      if (!mounted) return;
      appendLogs([event.payload]);
    }).then((fn) => {
      unlistenLog = fn;
    });

    listen<LogEvent[]>('log-batch', (event) => {
      if (!mounted) return;
      appendLogs(event.payload);
    }).then((fn) => {
      unlistenBatch = fn;
    });

    return () => {
      mounted = false;
      unlistenLog?.();
      unlistenBatch?.();
    };
  }, [terminalRef, shouldDisplayLog]);

  const clearLogs = useCallback(() => {
    logsRef.current = [];
    setLogs([]);
    terminalRef.current?.clear();
  }, [terminalRef]);

  return { logs, clearLogs };
}
