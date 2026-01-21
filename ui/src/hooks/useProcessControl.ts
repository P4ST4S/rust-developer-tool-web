import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useCallback, useEffect, useState } from 'react';
import type { StatusEvent, ServiceStatus } from '../types/events';

export function useProcessControl(projectId: string) {
  const [status, setStatus] = useState<Record<string, ServiceStatus>>({});
  const [loading, setLoading] = useState<Record<string, boolean>>({});

  useEffect(() => {
    invoke<StatusEvent>('get_status')
      .then((event) => setStatus(event.services))
      .catch(console.error);
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    listen<StatusEvent>('status-change', (event) => {
      setStatus(event.payload.services);
      setLoading({});
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const startService = useCallback(
    async (serviceId: string) => {
      const compositeId = `${projectId}:${serviceId}`;
      setLoading((prev) => ({ ...prev, [compositeId]: true }));
      try {
        await invoke('start_service', { projectId, serviceId });
      } catch (error) {
        console.error('Failed to start service:', error);
      } finally {
        setLoading((prev) => ({ ...prev, [compositeId]: false }));
      }
    },
    [projectId]
  );

  const stopService = useCallback(
    async (serviceId: string) => {
      const compositeId = `${projectId}:${serviceId}`;
      setLoading((prev) => ({ ...prev, [compositeId]: true }));
      try {
        await invoke('stop_service', { projectId, serviceId });
      } catch (error) {
        console.error('Failed to stop service:', error);
      } finally {
        setLoading((prev) => ({ ...prev, [compositeId]: false }));
      }
    },
    [projectId]
  );

  const openBrowser = useCallback(async (url: string) => {
    try {
      await invoke('open_browser', { url });
    } catch (error) {
      console.error('Failed to open browser:', error);
    }
  }, []);

  const getServiceStatus = useCallback(
    (serviceId: string): ServiceStatus => {
      const compositeId = `${projectId}:${serviceId}`;
      return status[compositeId] || { running: false, url: null };
    },
    [projectId, status]
  );

  const isServiceLoading = useCallback(
    (serviceId: string): boolean => {
      const compositeId = `${projectId}:${serviceId}`;
      return loading[compositeId] || false;
    },
    [projectId, loading]
  );

  return {
    status,
    loading,
    startService,
    stopService,
    openBrowser,
    getServiceStatus,
    isServiceLoading,
  };
}
