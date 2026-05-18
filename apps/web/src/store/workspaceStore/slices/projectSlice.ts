import { getExampleProject } from '../../../examples/catalog';
import { serializeCreasePatternSvg, renderCreasePatternPng } from '../../../lib/creaseExport';
import { createEmptyProject } from '../../../lib/sampleProject';
import { ensureExtension, getFileService, type FileService } from '../../../platform/fileService';
import { useLayoutStore } from '../../layoutStore';
import {
  createBlankTree,
  createStarterTree,
  engineError,
  ensureTreeHandle,
  getEngine,
  initializeBlankTree,
  loadTreeFromText,
  projectStateFromSnapshot,
  statusFromSnapshot,
} from '../engineRuntime';
import type { ProjectSlice, RecentProject, WorkspaceSliceCreator } from '../types';

const RECENTS_STORAGE_KEY = 'treemaker.recentProjects.v1';
const AUTOSAVE_STORAGE_KEY = 'treemaker.autosave.v1';
const MAX_RECENTS = 8;

function nowIso(): string {
  return new Date().toISOString();
}

function basenameWithoutTreeMakerExtension(filename: string): string {
  return filename.replace(/\.(tmd5?|tmd4)$/i, '') || 'Untitled';
}

function defaultFilename(title: string, extension: string): string {
  const base = title.trim() || 'Untitled';
  const safe = base.replace(/[^a-z0-9._-]+/gi, '-').replace(/^-+|-+$/g, '') || 'Untitled';
  return ensureExtension(safe, extension);
}

function loadRecentProjects(): RecentProject[] {
  if (typeof localStorage === 'undefined') return [];
  try {
    const parsed = JSON.parse(localStorage.getItem(RECENTS_STORAGE_KEY) ?? '[]') as RecentProject[];
    return Array.isArray(parsed) ? parsed.slice(0, MAX_RECENTS) : [];
  } catch {
    return [];
  }
}

function persistRecentProjects(recents: RecentProject[]): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(RECENTS_STORAGE_KEY, JSON.stringify(recents.slice(0, MAX_RECENTS)));
}

function persistAutosave(recent: RecentProject): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(AUTOSAVE_STORAGE_KEY, JSON.stringify(recent));
}

function confirmDiscardDirty(dirty: boolean): boolean {
  if (!dirty) return true;
  if (typeof window === 'undefined') return true;
  return window.confirm('Discard unsaved changes?');
}

export const createProjectSlice: WorkspaceSliceCreator<ProjectSlice> = (set, get) => {
  const rememberRecent = (recent: RecentProject) => {
    const existing = get().recentProjects;
    const next = [
      recent,
      ...existing.filter((item) => item.id !== recent.id && item.filename !== recent.filename),
    ].slice(0, MAX_RECENTS);
    persistRecentProjects(next);
    persistAutosave(recent);
    set({ recentProjects: next });
  };

  const loadText = async (
    text: string,
    source: { title?: string; filename?: string; path?: string | null; dirty?: boolean } = {}
  ) => {
    set({ status: 'loading_engine', error: null, projectMessage: null });
    const api = await getEngine();
    const snapshot = await loadTreeFromText(api, text);
    const filename = source.filename ?? 'Untitled.tmd5';
    const title = source.title ?? basenameWithoutTreeMakerExtension(filename);
    set({
      ...projectStateFromSnapshot(snapshot, title),
      currentFileName: filename,
      currentFilePath: source.path ?? null,
      projectMessage: `Loaded ${filename}`,
      selection: { kind: 'tree' },
      toolMode: 'select',
      creaseColorMode: 'mvf',
      foldArtifacts: null,
      foldArtifactError: null,
      status: statusFromSnapshot(snapshot),
      dirty: source.dirty ?? false,
      lastOptimization: null,
      historyPast: [],
      historyFuture: [],
      clipboardPasteCount: 0,
    });
    rememberRecent({
      id: source.path ?? filename,
      title,
      filename,
      savedAt: nowIso(),
      text,
    });
    useLayoutStore.getState().activatePanel('design');
  };

  const saveTmd5 = async (fileService: FileService, forceSaveAs: boolean) => {
    const { api, treeHandle, initializedSnapshot } = await ensureTreeHandle();
    if (initializedSnapshot) {
      set(projectStateFromSnapshot(initializedSnapshot, get().project.title));
    }
    const contents = await api.saveTmd5(treeHandle);
    const suggestedName = defaultFilename(get().project.title, 'tmd5');
    const result = await fileService.saveTextFile({
      title: forceSaveAs ? 'Save TreeMaker Project As' : 'Save TreeMaker Project',
      contents,
      suggestedName: get().currentFileName || suggestedName,
      path: forceSaveAs ? null : get().currentFilePath,
      extensions: ['tmd5'],
    });
    if (!result) return false;
    set({
      currentFileName: result.name,
      currentFilePath: result.path,
      dirty: false,
      projectMessage: `Saved ${result.name}`,
    });
    rememberRecent({
      id: result.path ?? result.name,
      title: get().project.title,
      filename: result.name,
      savedAt: nowIso(),
      text: contents,
    });
    return true;
  };

  return {
    project: createEmptyProject(),
    currentFilePath: null,
    currentFileName: 'Untitled.tmd5',
    projectMessage: null,
    recentProjects: loadRecentProjects(),
    status: 'loading_engine',
    dirty: false,
    engineReady: false,
    error: null,
    lastOptimization: null,

    initEngine: async () => {
      set({ status: 'loading_engine', error: null });
      try {
        const api = await getEngine();
        const snapshot = await initializeBlankTree(api);
        set({
          ...projectStateFromSnapshot(snapshot, get().project.title),
          selection: { kind: 'tree' },
          dirty: false,
          lastOptimization: null,
          foldArtifacts: null,
          foldArtifactError: null,
          historyPast: [],
          historyFuture: [],
        });
      } catch (error) {
        set({ status: 'error', error: engineError(error), engineReady: false });
      }
    },

    createNewProject: async () => {
      if (!confirmDiscardDirty(get().dirty)) return;
      set({ status: 'loading_engine', error: null, projectMessage: null });
      try {
        const api = await getEngine();
        const snapshot = await createBlankTree(api);
        set({
          ...projectStateFromSnapshot(snapshot, 'Untitled'),
          currentFileName: 'Untitled.tmd5',
          currentFilePath: null,
          projectMessage: null,
          selection: { kind: 'tree' },
          toolMode: 'select',
          creaseColorMode: 'mvf',
          foldArtifacts: null,
          foldArtifactError: null,
          dirty: false,
          lastOptimization: null,
          historyPast: [],
          historyFuture: [],
          clipboardPasteCount: 0,
        });
        useLayoutStore.getState().activatePanel('design');
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    loadStarterProject: async () => {
      if (!confirmDiscardDirty(get().dirty)) return;
      set({ status: 'loading_engine', error: null, projectMessage: null });
      try {
        const api = await getEngine();
        const snapshot = await createStarterTree(api);
        set({
          ...projectStateFromSnapshot(snapshot, 'Three terminal flaps'),
          currentFileName: 'three-terminal-flaps.tmd5',
          currentFilePath: null,
          projectMessage: 'Loaded starter project',
          selection: { kind: 'tree' },
          toolMode: 'select',
          creaseColorMode: 'mvf',
          foldArtifacts: null,
          foldArtifactError: null,
          dirty: false,
          lastOptimization: null,
          historyPast: [],
          historyFuture: [],
          clipboardPasteCount: 0,
        });
        useLayoutStore.getState().activatePanel('design');
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
      }
    },

    loadProjectText: async (text, source) => {
      try {
        await loadText(text, source);
      } catch (error) {
        set({ status: 'error', error: engineError(error), projectMessage: null });
      }
    },

    openProject: async (fileService = getFileService()) => {
      if (!confirmDiscardDirty(get().dirty)) return false;
      try {
        const file = await fileService.openTextFile({
          title: 'Open TreeMaker Project',
          extensions: ['tmd', 'tmd4', 'tmd5'],
        });
        if (!file) return false;
        await loadText(file.text, { filename: file.name, path: file.path });
        return true;
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
        return false;
      }
    },

    saveProject: async (fileService = getFileService()) => {
      try {
        return await saveTmd5(fileService, false);
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
        return false;
      }
    },

    saveProjectAs: async (fileService = getFileService()) => {
      try {
        return await saveTmd5(fileService, true);
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
        return false;
      }
    },

    exportV4: async (fileService = getFileService()) => {
      try {
        const { api, treeHandle } = await ensureTreeHandle();
        const contents = await api.exportV4(treeHandle);
        const result = await fileService.saveTextFile({
          title: 'Export TreeMaker 4 Project',
          contents,
          suggestedName: defaultFilename(get().project.title, 'tmd4'),
          path: null,
          extensions: ['tmd4'],
        });
        if (!result) return false;
        set({ projectMessage: `Exported ${result.name}` });
        return true;
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
        return false;
      }
    },

    exportFold: async (fileService = getFileService()) => {
      try {
        const { api, treeHandle } = await ensureTreeHandle();
        const contents = await api.exportFold(treeHandle);
        const result = await fileService.saveTextFile({
          title: 'Export FOLD Document',
          contents,
          suggestedName: defaultFilename(get().project.title, 'fold'),
          path: null,
          extensions: ['fold'],
        });
        if (!result) return false;
        set({ projectMessage: `Exported ${result.name}` });
        return true;
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
        return false;
      }
    },

    exportSvg: async (fileService = getFileService()) => {
      try {
        const contents = serializeCreasePatternSvg(get().project, get().creaseColorMode);
        const result = await fileService.saveTextFile({
          title: 'Export Crease Pattern SVG',
          contents,
          suggestedName: defaultFilename(get().project.title, 'svg'),
          path: null,
          extensions: ['svg'],
        });
        if (!result) return false;
        set({ projectMessage: `Exported ${result.name}` });
        return true;
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
        return false;
      }
    },

    exportPng: async (fileService = getFileService()) => {
      try {
        const bytes = await renderCreasePatternPng(get().project, get().creaseColorMode);
        const result = await fileService.saveBinaryFile({
          title: 'Export Crease Pattern PNG',
          bytes,
          suggestedName: defaultFilename(get().project.title, 'png'),
          path: null,
          extensions: ['png'],
          mimeType: 'image/png',
        });
        if (!result) return false;
        set({ projectMessage: `Exported ${result.name}` });
        return true;
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
        return false;
      }
    },

    loadExampleProject: async (id) => {
      if (!confirmDiscardDirty(get().dirty)) return;
      const example = getExampleProject(id);
      if (!example) return;
      await get().loadProjectText(example.text, {
        title: example.title,
        filename: example.filename,
      });
    },

    loadRecentProject: async (id) => {
      if (!confirmDiscardDirty(get().dirty)) return;
      const recent = get().recentProjects.find((item) => item.id === id);
      if (!recent) return;
      await get().loadProjectText(recent.text, {
        title: recent.title,
        filename: recent.filename,
      });
    },

    autosaveProject: async () => {
      if (!get().dirty) return;
      try {
        const { api, treeHandle } = await ensureTreeHandle();
        const text = await api.saveTmd5(treeHandle);
        rememberRecent({
          id: AUTOSAVE_STORAGE_KEY,
          title: get().project.title,
          filename: get().currentFileName,
          savedAt: nowIso(),
          text,
        });
      } catch {
        // Autosave is best-effort and should never interrupt editing.
      }
    },

    clearProjectMessage: () => set({ projectMessage: null }),
  };
};
