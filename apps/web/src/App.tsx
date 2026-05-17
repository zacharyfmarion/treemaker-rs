import { useCallback } from 'react';
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
import { applyDefaultLayout, useLayoutStore } from './store/layoutStore';
import { useWorkspaceStore } from './store/workspaceStore';

function Toolbar() {
  const createNewProject = useWorkspaceStore((state) => state.createNewProject);
  const status = useWorkspaceStore((state) => state.status);
  const dirty = useWorkspaceStore((state) => state.dirty);

  return (
    <header className="toolbar">
      <div className="toolbar__brand">
        <span className="toolbar__title">TreeMaker</span>
        <span className="toolbar__status" data-status={status}>
          {status.replaceAll('_', ' ')}
        </span>
        {dirty && <span className="toolbar__dirty">Unsaved</span>}
      </div>
      <div className="toolbar__actions">
        <IconButton size="sm" title="New" tooltipSide="bottom" onClick={createNewProject}>
          <FilePlus size={15} />
        </IconButton>
        <IconButton size="sm" title="Open" tooltipSide="bottom" disabled>
          <FolderOpen size={15} />
        </IconButton>
        <IconButton size="sm" title="Save" tooltipSide="bottom" disabled>
          <Save size={15} />
        </IconButton>
        <span className="toolbar__separator" />
        <Button size="sm" variant="secondary" disabled>
          <Sparkles size={14} />
          Optimize
        </Button>
        <Button size="sm" variant="primary" disabled>
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
  const setDockviewApi = useLayoutStore((state) => state.setDockviewApi);
  const loadLayout = useLayoutStore((state) => state.loadLayout);
  const saveLayout = useLayoutStore((state) => state.saveLayout);

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
