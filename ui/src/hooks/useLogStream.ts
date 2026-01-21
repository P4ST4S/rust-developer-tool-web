import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useEffect, useRef, useCallback } from 'react';
import type { LogEvent, Filters } from '../types/events';
import type { TerminalHandle } from '../components/Terminal';

const MAX_LOGS = 20000;
const REWRITE_CHUNK_SIZE = 500;

type LogBuffer = {
  items: LogEvent[];
  start: number;
  size: number;
  capacity: number;
};

const createLogBuffer = (capacity: number): LogBuffer => ({
  items: [],
  start: 0,
  size: 0,
  capacity,
});

const pushLog = (buffer: LogBuffer, log: LogEvent) => {
  if (buffer.size < buffer.capacity) {
    buffer.items.push(log);
    buffer.size += 1;
    return;
  }

  buffer.items[buffer.start] = log;
  buffer.start = (buffer.start + 1) % buffer.capacity;
};

const getLogAt = (buffer: LogBuffer, index: number): LogEvent | undefined => {
  if (index < 0 || index >= buffer.size || buffer.size === 0) {
    return undefined;
  }

  if (buffer.size < buffer.capacity) {
    return buffer.items[index];
  }

  const itemIndex = (buffer.start + index) % buffer.capacity;
  return buffer.items[itemIndex];
};

export function useLogStream(
  terminalRef: React.RefObject<TerminalHandle | null>,
  filters: Filters,
  projectId: string
) {
  const logBufferRef = useRef<LogBuffer>(createLogBuffer(MAX_LOGS));
  const rewriteTokenRef = useRef(0);
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

    const buffer = logBufferRef.current;
    terminalRef.current.clear();
    if (buffer.size === 0) return;

    const token = rewriteTokenRef.current + 1;
    rewriteTokenRef.current = token;

    let index = 0;
    const total = buffer.size;

    const writeChunk = () => {
      if (rewriteTokenRef.current !== token) return;
      if (!terminalRef.current) return;

      let written = 0;
      while (index < total && written < REWRITE_CHUNK_SIZE) {
        const log = getLogAt(buffer, index);
        if (log && shouldDisplayLog(log)) {
          terminalRef.current.writeln(log.text);
        }
        index += 1;
        written += 1;
      }

      if (index < total) {
        if (typeof requestAnimationFrame === 'function') {
          requestAnimationFrame(writeChunk);
        } else {
          setTimeout(writeChunk, 0);
        }
      }
    };

    writeChunk();
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

      for (const newLog of incomingLogs) {
        if (newLog.project_id !== projectIdRef.current) {
          continue;
        }

        pushLog(logBufferRef.current, newLog);
        if (shouldDisplayLog(newLog) && terminalRef.current) {
          terminalRef.current.writeln(newLog.text);
        }
      }
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
    logBufferRef.current = createLogBuffer(MAX_LOGS);
    terminalRef.current?.clear();
  }, [terminalRef]);

  return { clearLogs };
}
