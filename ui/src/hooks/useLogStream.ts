import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useEffect, useRef, useCallback, useState } from 'react';
import type { LogEvent, Filters } from '../types/events';
import type { TerminalHandle } from '../components/Terminal';

const MAX_LOGS = 20000;

export function useLogStream(
  terminalRef: React.RefObject<TerminalHandle | null>,
  filters: Filters
) {
  const [logs, setLogs] = useState<LogEvent[]>([]);
  const logsRef = useRef<LogEvent[]>([]);

  const shouldDisplayLog = useCallback(
    (log: LogEvent): boolean => {
      if (filters.source !== 'all' && filters.source !== log.source) {
        return false;
      }
      if (filters.level !== 'all' && filters.level !== log.level) {
        return false;
      }
      return true;
    },
    [filters.source, filters.level]
  );

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

    listen<LogEvent>('log', (event) => {
      const newLog = event.payload;

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
      unlisten?.();
    };
  }, [terminalRef, shouldDisplayLog]);

  const clearLogs = useCallback(() => {
    logsRef.current = [];
    setLogs([]);
    terminalRef.current?.clear();
  }, [terminalRef]);

  return { logs, clearLogs };
}
