import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useCallback, useEffect, useState } from 'react';
import type { StatusEvent } from '../types/events';

export function useProcessControl() {
  const [status, setStatus] = useState<StatusEvent>({
    frontend_running: false,
    backend_running: false,
    frontend_url: null,
  });
  const [loading, setLoading] = useState({
    frontend: false,
    backend: false,
  });

  useEffect(() => {
    invoke<StatusEvent>('get_status').then(setStatus).catch(console.error);
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    listen<StatusEvent>('status-change', (event) => {
      setStatus(event.payload);
      setLoading({ frontend: false, backend: false });
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const startFrontend = useCallback(async () => {
    setLoading((prev) => ({ ...prev, frontend: true }));
    try {
      await invoke('start_frontend');
      setLoading((prev) => ({ ...prev, frontend: false }));
    } catch (error) {
      console.error('Failed to start frontend:', error);
      setLoading((prev) => ({ ...prev, frontend: false }));
    }
  }, []);

  const stopFrontend = useCallback(async () => {
    setLoading((prev) => ({ ...prev, frontend: true }));
    try {
      await invoke('stop_frontend');
      setLoading((prev) => ({ ...prev, frontend: false }));
    } catch (error) {
      console.error('Failed to stop frontend:', error);
      setLoading((prev) => ({ ...prev, frontend: false }));
    }
  }, []);

  const startBackend = useCallback(async () => {
    setLoading((prev) => ({ ...prev, backend: true }));
    try {
      await invoke('start_backend');
      setLoading((prev) => ({ ...prev, backend: false }));
    } catch (error) {
      console.error('Failed to start backend:', error);
      setLoading((prev) => ({ ...prev, backend: false }));
    }
  }, []);

  const stopBackend = useCallback(async () => {
    setLoading((prev) => ({ ...prev, backend: true }));
    try {
      await invoke('stop_backend');
      setLoading((prev) => ({ ...prev, backend: false }));
    } catch (error) {
      console.error('Failed to stop backend:', error);
      setLoading((prev) => ({ ...prev, backend: false }));
    }
  }, []);

  const openBrowser = useCallback(async () => {
    if (status.frontend_url) {
      try {
        await invoke('open_browser', { url: status.frontend_url });
      } catch (error) {
        console.error('Failed to open browser:', error);
      }
    }
  }, [status.frontend_url]);

  return {
    status,
    loading,
    startFrontend,
    stopFrontend,
    startBackend,
    stopBackend,
    openBrowser,
  };
}
