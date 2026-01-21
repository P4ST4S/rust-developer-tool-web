import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useState } from 'react';
import type { Config, Project, Service } from '../types/events';

export function useConfig() {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<Config | null>('get_config')
      .then((cfg) => {
        setConfig(cfg);
        setLoading(false);
      })
      .catch((err) => {
        console.error('Failed to load config:', err);
        setLoading(false);
      });
  }, []);

  const saveConfig = useCallback(async (newConfig: Config) => {
    try {
      await invoke('save_app_config', { config: newConfig });
      setConfig(newConfig);
    } catch (err) {
      console.error('Failed to save config:', err);
      throw err;
    }
  }, []);

  const addProject = useCallback(
    async (project: Project) => {
      if (!config) {
        const newConfig: Config = {
          version: 1,
          active_project: project.id,
          projects: [project],
        };
        await saveConfig(newConfig);
      } else {
        const newConfig: Config = {
          ...config,
          projects: [...config.projects, project],
          active_project: config.active_project || project.id,
        };
        await saveConfig(newConfig);
      }
    },
    [config, saveConfig]
  );

  const updateProject = useCallback(
    async (projectId: string, updates: Partial<Project>) => {
      if (!config) return;
      const newConfig: Config = {
        ...config,
        projects: config.projects.map((p) =>
          p.id === projectId ? { ...p, ...updates } : p
        ),
      };
      await saveConfig(newConfig);
    },
    [config, saveConfig]
  );

  const deleteProject = useCallback(
    async (projectId: string) => {
      if (!config) return;
      const newProjects = config.projects.filter((p) => p.id !== projectId);
      const newConfig: Config = {
        ...config,
        projects: newProjects,
        active_project:
          config.active_project === projectId
            ? newProjects[0]?.id || null
            : config.active_project,
      };
      await saveConfig(newConfig);
    },
    [config, saveConfig]
  );

  const addService = useCallback(
    async (projectId: string, service: Service) => {
      if (!config) return;
      const newConfig: Config = {
        ...config,
        projects: config.projects.map((p) =>
          p.id === projectId
            ? { ...p, services: [...p.services, service] }
            : p
        ),
      };
      await saveConfig(newConfig);
    },
    [config, saveConfig]
  );

  const updateService = useCallback(
    async (projectId: string, serviceId: string, updates: Partial<Service>) => {
      if (!config) return;
      const newConfig: Config = {
        ...config,
        projects: config.projects.map((p) =>
          p.id === projectId
            ? {
                ...p,
                services: p.services.map((s) =>
                  s.id === serviceId ? { ...s, ...updates } : s
                ),
              }
            : p
        ),
      };
      await saveConfig(newConfig);
    },
    [config, saveConfig]
  );

  const deleteService = useCallback(
    async (projectId: string, serviceId: string) => {
      if (!config) return;
      const newConfig: Config = {
        ...config,
        projects: config.projects.map((p) =>
          p.id === projectId
            ? { ...p, services: p.services.filter((s) => s.id !== serviceId) }
            : p
        ),
      };
      await saveConfig(newConfig);
    },
    [config, saveConfig]
  );

  const setActiveProject = useCallback(
    async (projectId: string) => {
      try {
        await invoke('set_active_project', { projectId });
        setConfig((prev) =>
          prev ? { ...prev, active_project: projectId } : prev
        );
      } catch (err) {
        console.error('Failed to set active project:', err);
      }
    },
    []
  );

  return {
    config,
    loading,
    saveConfig,
    addProject,
    updateProject,
    deleteProject,
    addService,
    updateService,
    deleteService,
    setActiveProject,
  };
}
