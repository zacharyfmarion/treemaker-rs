import { useCallback, useEffect } from 'react';
import { DockviewReact } from 'dockview';
import type { DockviewReadyEvent } from 'dockview';
import 'dockview/dist/styles/dockview.css';
import {
  FilePlus,
  FolderOpen,
  Save,
  Settings,
  Sparkles,
  Play,
} from 'lucide-react';
import { TooltipProvider } from './components/ui/Tooltip';
import { IconButton } from './components/ui/IconButton';
import { Button } from './components/ui/Button';
import { panelComponents } from './components/panels/PanelComponents';
import { handleMenuAction } from './commands/menuActions';
import { useTauriMenuListener } from './menus/tauriMenuListener';
import { applyWindowTitle, formatWindowTitle } from './platform/windowTitle';
import { applyDefaultLayout, useLayoutStore } from './store/layoutStore';
import { useWorkspaceStore } from './store/workspaceStore';

function Toolbar() {
  const status = useWorkspaceStore((state) => state.status);
  const project = useWorkspaceStore((state) => state.project);
  const engineReady = useWorkspaceStore((state) => state.engineReady);
  const error = useWorkspaceStore((state) => state.error);
  const dirty = useWorkspaceStore((state) => state.dirty);
  const busy =
    status === 'loading_engine' ||
    status === 'optimizing' ||
    status === 'building_crease_pattern';
  const canOptimize = engineReady && project.edges.length > 0 && !busy && status !== 'error';
  const canBuild =
    engineReady && !busy && (status === 'optimized' || status === 'crease_pattern_ready');

  return (
    <header className="toolbar">
      <div className="toolbar__brand">
        <span className="toolbar__title">TreeMaker</span>
        <span className="toolbar__status" data-status={status}>
          {status.replaceAll('_', ' ')}
        </span>
        {dirty && <span className="toolbar__dirty">Unsaved</span>}
        {error && (
          <span className="toolbar__error" title={error.message}>
            {error.message}
          </span>
        )}
      </div>
      <div className="toolbar__actions">
        <IconButton
          size="sm"
          title="New"
          tooltipSide="bottom"
          disabled={busy}
          onClick={() => void handleMenuAction('file.new')}
        >
          <FilePlus size={15} />
        </IconButton>
        <IconButton
          size="sm"
          title="Open"
          tooltipSide="bottom"
          onClick={() => void handleMenuAction('file.open')}
        >
          <FolderOpen size={15} />
        </IconButton>
        <IconButton
          size="sm"
          title="Save"
          tooltipSide="bottom"
          onClick={() => void handleMenuAction('file.save')}
        >
          <Save size={15} />
        </IconButton>
        <span className="toolbar__separator" />
        <Button
          size="sm"
          variant="secondary"
          disabled={!canOptimize}
          onClick={() => void handleMenuAction('optimize.scale')}
        >
          <Sparkles size={14} />
          Optimize
        </Button>
        <Button
          size="sm"
          variant="primary"
          disabled={!canBuild}
          onClick={() => void handleMenuAction('cp.build')}
        >
          <Play size={14} />
          Build CP
        </Button>
        <span className="toolbar__separator" />
        <IconButton size="sm" title="Settings" tooltipSide="bottom" disabled>
          <Settings size={15} />
        </IconButton>
      </div>
    </header>
  );
}

export default function App() {
  const initEngine = useWorkspaceStore((state) => state.initEngine);
  const deleteSelection = useWorkspaceStore((state) => state.deleteSelection);
  const project = useWorkspaceStore((state) => state.project);
  const dirty = useWorkspaceStore((state) => state.dirty);
  const setDockviewApi = useLayoutStore((state) => state.setDockviewApi);
  const loadLayout = useLayoutStore((state) => state.loadLayout);
  const saveLayout = useLayoutStore((state) => state.saveLayout);

  useTauriMenuListener();

  useEffect(() => {
    void initEngine();
  }, [initEngine]);

  useEffect(() => {
    const title = formatWindowTitle({ projectTitle: project.title, dirty });
    void applyWindowTitle(title);
  }, [dirty, project.title]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target;
      const isEditing =
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target instanceof HTMLSelectElement;
      if (isEditing || (event.key !== 'Delete' && event.key !== 'Backspace')) return;
      event.preventDefault();
      void deleteSelection();
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [deleteSelection]);

  const onReady = useCallback(
    (event: DockviewReadyEvent) => {
      const { api } = event;
      setDockviewApi(api);

      let loaded = false;
      const saved = loadLayout();
      if (saved) {
        try {
          api.fromJSON(saved);
          loaded = true;
        } catch (error) {
          console.warn('Failed to restore layout', error);
          localStorage.removeItem('treemaker-web-layout');
          localStorage.removeItem('treemaker-web-layout-version');
        }
      }

      if (!loaded) {
        applyDefaultLayout(api);
      }

      let timer: ReturnType<typeof setTimeout> | null = null;
      api.onDidLayoutChange(() => {
        if (timer) clearTimeout(timer);
        timer = setTimeout(() => saveLayout(), 250);
      });
    },
    [loadLayout, saveLayout, setDockviewApi]
  );

  return (
    <TooltipProvider>
      <div className="app-layout">
        <Toolbar />
        <DockviewReact
          components={panelComponents}
          onReady={onReady}
          className="dockview-theme-treemaker"
          disableFloatingGroups
        />
      </div>
    </TooltipProvider>
  );
}
