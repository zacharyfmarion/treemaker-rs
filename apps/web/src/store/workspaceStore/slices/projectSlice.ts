import { getExampleProject } from '../../../examples/catalog';
import { serializeCreasePatternSvg, renderCreasePatternPng } from '../../../lib/creaseExport';
import {
  importedCreasePatternFormat,
  isCreasePatternFilename,
  parseImportedCreasePattern,
  withFlatFoldArtifacts,
  withFlatFoldError,
} from '../../../lib/creasePatternImport';
import { createEmptyProject, DEFAULT_CREASE_COLOR_MODE } from '../../../lib/sampleProject';
import {
  getWorkspaceCapabilities,
  type WorkspaceCapabilityId,
} from '../../../lib/workspaceCapabilities';
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

  const capabilities = () =>
    getWorkspaceCapabilities({
      documentMode: get().documentMode,
      engineReady: get().engineReady,
      status: get().status,
      edgeCount: get().project.edges.length,
      creaseCount: get().project.creases.length,
      facetCount: get().project.facets.length,
      hasImportedCreasePattern: get().importedCreasePattern !== null,
      hasSimulationModel: get().foldArtifacts?.simulation_model != null,
      historyPastCount: get().historyPast.length,
      historyFutureCount: get().historyFuture.length,
      clipboard: get().clipboard,
      selection: get().selection,
    });

  const rejectDisabled = (id: WorkspaceCapabilityId) => {
    const capability = capabilities()[id];
    if (capability.enabled) return false;
    set({
      error: { code: 'invalid_operation', message: capability.reason },
      projectMessage: null,
    });
    return true;
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
      documentMode: 'tree',
      importedCreasePattern: null,
      projectLoadId: get().projectLoadId + 1,
      currentFileName: filename,
      currentFilePath: source.path ?? null,
      projectMessage: `Loaded ${filename}`,
      selection: { kind: 'tree' },
      toolMode: 'select',
      symmetryAuthoringPairs: [],
      creaseColorMode: DEFAULT_CREASE_COLOR_MODE,
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

  const loadCreasePattern = async (
    text: string,
    source: { filename: string; path?: string | null }
  ) => {
    set({ status: 'loading_engine', error: null, projectMessage: null });
    const filename = source.filename;
    const parsed = parseImportedCreasePattern(text, {
      format: importedCreasePatternFormat(filename),
      filename,
      path: source.path ?? null,
    });
    const result = await (async () => {
      try {
        const api = await getEngine();
        const foldArtifacts = await api.flatFoldArtifacts(JSON.stringify(parsed.document.fold), {
          solution_limit: 10,
        });
        return withFlatFoldArtifacts(parsed, foldArtifacts);
      } catch (error) {
        return withFlatFoldError(parsed, engineError(error).message);
      }
    })();
    set({
      project: result.project,
      documentMode: 'crease-pattern',
      importedCreasePattern: result.document,
      projectLoadId: get().projectLoadId + 1,
      currentFileName: filename,
      currentFilePath: source.path ?? null,
      projectMessage: `Loaded ${filename}`,
      selection: { kind: 'tree' },
      toolMode: 'select',
      creaseColorMode: DEFAULT_CREASE_COLOR_MODE,
      foldArtifacts: result.foldArtifacts,
      foldArtifactError: null,
      status: 'crease_pattern_ready',
      dirty: false,
      error: null,
      engineReady: true,
      lastOptimization: null,
      historyPast: [],
      historyFuture: [],
      clipboardPasteCount: 0,
    });
    rememberRecent({
      id: source.path ?? filename,
      title: result.document.title,
      filename,
      savedAt: nowIso(),
      text,
    });
    useLayoutStore.getState().activatePanel('crease-pattern');
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
    documentMode: 'tree',
    importedCreasePattern: null,
    projectLoadId: 0,
    currentFilePath: null,
    currentFileName: 'Untitled.tmd5',
    projectMessage: null,
    recentProjects: loadRecentProjects(),
    status: 'loading_engine',
    dirty: false,
    engineReady: false,
    error: null,
    lastOptimization: null,
    designViewportFitRequestId: 0,

    initEngine: async () => {
      set({ status: 'loading_engine', error: null });
      try {
        const api = await getEngine();
        const snapshot = await initializeBlankTree(api);
        if (get().documentMode !== 'tree') {
          set({ engineReady: true });
          return;
        }
        set({
          ...projectStateFromSnapshot(snapshot, get().project.title),
          documentMode: 'tree',
          importedCreasePattern: null,
          projectLoadId: get().projectLoadId + 1,
          selection: { kind: 'tree' },
          symmetryAuthoringPairs: [],
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
      if (rejectDisabled('file.new')) return;
      if (!confirmDiscardDirty(get().dirty)) return;
      set({ status: 'loading_engine', error: null, projectMessage: null });
      try {
        const api = await getEngine();
        const snapshot = await createBlankTree(api);
        set({
          ...projectStateFromSnapshot(snapshot, 'Untitled'),
          documentMode: 'tree',
          importedCreasePattern: null,
          projectLoadId: get().projectLoadId + 1,
          currentFileName: 'Untitled.tmd5',
          currentFilePath: null,
          projectMessage: null,
          selection: { kind: 'tree' },
          toolMode: 'select',
          symmetryAuthoringPairs: [],
          creaseColorMode: DEFAULT_CREASE_COLOR_MODE,
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
          documentMode: 'tree',
          importedCreasePattern: null,
          projectLoadId: get().projectLoadId + 1,
          currentFileName: 'three-terminal-flaps.tmd5',
          currentFilePath: null,
          projectMessage: 'Loaded starter project',
          selection: { kind: 'tree' },
          toolMode: 'select',
          symmetryAuthoringPairs: [],
          creaseColorMode: DEFAULT_CREASE_COLOR_MODE,
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

    loadCreasePatternText: async (text, source) => {
      try {
        await loadCreasePattern(text, source);
      } catch (error) {
        set({ status: 'error', error: engineError(error), projectMessage: null });
      }
    },

    openProject: async (fileService = getFileService()) => {
      if (rejectDisabled('file.open')) return false;
      if (!confirmDiscardDirty(get().dirty)) return false;
      try {
        const file = await fileService.openTextFile({
          title: 'Open TreeMaker Project or Crease Pattern',
          extensions: ['tmd', 'tmd4', 'tmd5', 'fold', 'cp'],
        });
        if (!file) return false;
        if (isCreasePatternFilename(file.name)) {
          await loadCreasePattern(file.text, { filename: file.name, path: file.path });
        } else {
          await loadText(file.text, { filename: file.name, path: file.path });
        }
        return true;
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
        return false;
      }
    },

    saveProject: async (fileService = getFileService()) => {
      try {
        if (rejectDisabled('file.save')) return false;
        return await saveTmd5(fileService, false);
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
        return false;
      }
    },

    saveProjectAs: async (fileService = getFileService()) => {
      try {
        if (rejectDisabled('file.saveAs')) return false;
        return await saveTmd5(fileService, true);
      } catch (error) {
        set({ status: 'error', error: engineError(error) });
        return false;
      }
    },

    exportV4: async (fileService = getFileService()) => {
      try {
        if (rejectDisabled('file.exportV4')) return false;
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
        if (rejectDisabled('file.exportFold')) return false;
        const contents =
          get().documentMode === 'crease-pattern' && get().importedCreasePattern
            ? JSON.stringify(get().importedCreasePattern?.fold, null, 2)
            : await (async () => {
                const { api, treeHandle } = await ensureTreeHandle();
                return api.exportFold(treeHandle);
              })();
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
        if (rejectDisabled('file.exportSvg')) return false;
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
        if (rejectDisabled('file.exportPng')) return false;
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
      if (isCreasePatternFilename(recent.filename)) {
        await get().loadCreasePatternText(recent.text, {
          filename: recent.filename,
          path: recent.id,
        });
      } else {
        await get().loadProjectText(recent.text, {
          title: recent.title,
          filename: recent.filename,
        });
      }
    },

    autosaveProject: async () => {
      if (get().documentMode !== 'tree') return;
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
