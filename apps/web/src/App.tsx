import { useCallback, useEffect } from 'react';
import { DockviewDefaultTab, DockviewReact } from 'dockview';
import type { DockviewReadyEvent, IDockviewPanelHeaderProps } from 'dockview';
import 'dockview/dist/styles/dockview.css';
import { Toaster } from 'sonner';
import {
  Download,
  FilePlus,
  FolderOpen,
  CircleHelp,
  Save,
  Settings,
  Sparkles,
  Play,
} from 'lucide-react';
import { MenuBar } from './components/MenuBar';
import { CommandDialogModal } from './components/CommandDialogModal';
import { GlobalToasts } from './components/GlobalToasts';
import { HelpModal } from './components/HelpModal';
import { SelectByIndexModal } from './components/SelectByIndexModal';
import { SettingsModal } from './components/SettingsModal';
import { TooltipProvider } from './components/ui/Tooltip';
import { IconButton } from './components/ui/IconButton';
import { Button } from './components/ui/Button';
import { panelComponents } from './components/panels/PanelComponents';
import { handleMenuAction } from './commands/menuActions';
import { useMacDownloadUrl } from './hooks/useMacDownloadUrl';
import { handleAppKeyDown } from './lib/appKeyboard';
import { cpSelectionSize } from './lib/creasePatternViewport';
import { useTauriMenuListener } from './menus/tauriMenuListener';
import { isFeatureVisible } from './platform/features';
import { getRuntimeSurface } from './platform/runtime';
import { applyWindowTitle, formatWindowTitle } from './platform/windowTitle';
import { requestConfirmation } from './store/commandDialogStore';
import { applyDefaultLayout, useLayoutStore } from './store/layoutStore';
import { useSettingsStore } from './store/settingsStore';
import { useThemeStore } from './store/themeStore';
import { useWorkspaceStore } from './store/workspaceStore';
import { useWorkspaceCapabilities } from './store/workspaceStore/useWorkspaceCapabilities';
import './styles/sonner.css';

function Toolbar() {
  const openSettings = useSettingsStore((state) => state.openSettings);
  const capabilities = useWorkspaceCapabilities();
  const runtimeSurface = getRuntimeSurface();
  const isDesktop = runtimeSurface === 'desktop';
  const showDownloadCta = isFeatureVisible('macDownloadCta', runtimeSurface);
  const downloadUrl = useMacDownloadUrl();
  const optimizeScale = capabilities['optimize.scale'];
  const buildCp = capabilities['cp.build'];

  return (
    <header className="toolbar">
      <div className="toolbar__brand">
        {isDesktop ? <span className="toolbar__title">Ori Studio</span> : <MenuBar />}
      </div>
      <div className="toolbar__actions">
        <IconButton
          size="sm"
          title="New"
          tooltipSide="bottom"
          disabled={!capabilities['file.new'].enabled}
          onClick={() => void handleMenuAction('file.new')}
        >
          <FilePlus size={15} />
        </IconButton>
        <IconButton
          size="sm"
          title="Open"
          tooltipSide="bottom"
          disabled={!capabilities['file.open'].enabled}
          onClick={() => void handleMenuAction('file.open')}
        >
          <FolderOpen size={15} />
        </IconButton>
        <IconButton
          size="sm"
          title="Save"
          tooltipSide="bottom"
          disabled={!capabilities['file.save'].enabled}
          onClick={() => void handleMenuAction('file.save')}
        >
          <Save size={15} />
        </IconButton>
        <span className="toolbar__separator" />
        {optimizeScale.visible && (
          <Button
            size="sm"
            variant={buildCp.enabled ? 'secondary' : 'primary'}
            disabled={!optimizeScale.enabled}
            title={optimizeScale.reason}
            onClick={() => void handleMenuAction('optimize.scale')}
          >
            <Sparkles size={14} />
            Optimize Scale
          </Button>
        )}
        {buildCp.visible && (
          <Button
            size="sm"
            variant={buildCp.enabled ? 'primary' : 'secondary'}
            disabled={!buildCp.enabled}
            title={buildCp.reason}
            onClick={() => void handleMenuAction('cp.build')}
          >
            <Play size={14} />
            {buildCp.label}
          </Button>
        )}
        {(optimizeScale.visible || buildCp.visible) && <span className="toolbar__separator" />}
        {showDownloadCta && (
          <IconButton
            size="sm"
            title="Download Ori Studio for Mac"
            tooltipSide="bottom"
            onClick={() => window.open(downloadUrl, '_blank', 'noreferrer')}
          >
            <Download size={15} />
          </IconButton>
        )}
        <IconButton
          size="sm"
          title="Help"
          tooltipSide="bottom"
          onClick={() => void handleMenuAction('help.documentation')}
        >
          <CircleHelp size={15} />
        </IconButton>
        <IconButton size="sm" title="Settings" tooltipSide="bottom" onClick={() => openSettings()}>
          <Settings size={15} />
        </IconButton>
      </div>
    </header>
  );
}

function FixedDockTab(props: IDockviewPanelHeaderProps) {
  return <DockviewDefaultTab {...props} hideClose />;
}

export default function App() {
  const initEngine = useWorkspaceStore((state) => state.initEngine);
  const selectNone = useWorkspaceStore((state) => state.selectNone);
  const project = useWorkspaceStore((state) => state.project);
  const dirty = useWorkspaceStore((state) => state.dirty);
  const toasterTheme = useThemeStore((state) => state.currentTheme.type);
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
    const onBeforeUnload = (event: BeforeUnloadEvent) => {
      if (!useWorkspaceStore.getState().dirty) return;
      event.preventDefault();
      event.returnValue = '';
    };
    window.addEventListener('beforeunload', onBeforeUnload);
    return () => window.removeEventListener('beforeunload', onBeforeUnload);
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    if (getRuntimeSurface() !== 'desktop') return undefined;
    import('@tauri-apps/api/window')
      .then(({ getCurrentWindow }) => {
        const appWindow = getCurrentWindow();
        return appWindow.onCloseRequested((event) => {
          if (!useWorkspaceStore.getState().dirty) return;
          event.preventDefault();
          void requestConfirmation({
            title: 'Discard unsaved changes?',
            message: 'Your current project has unsaved changes. Close Ori Studio and discard them?',
            confirmLabel: 'Discard',
            tone: 'danger',
          }).then((confirmed) => {
            if (confirmed) void appWindow.destroy();
          });
        });
      })
      .then((dispose) => {
        unlisten = dispose;
      })
      .catch((error) => {
        console.warn('Failed to register Tauri close guard', error);
      });

    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      handleAppKeyDown(event, {
        getDocumentMode: () => useWorkspaceStore.getState().documentMode,
        getCpSelectionSize: () =>
          cpSelectionSize(useWorkspaceStore.getState().oristudioCpSelection),
        getSelection: () => useWorkspaceStore.getState().selection,
        handleMenuAction,
        selectNone,
      });
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [selectNone]);

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
          defaultTabComponent={FixedDockTab}
          onReady={onReady}
          className="dockview-theme-treemaker"
          disableFloatingGroups
        />
      </div>
      <HelpModal />
      <SelectByIndexModal />
      <SettingsModal />
      <CommandDialogModal />
      <GlobalToasts />
      <Toaster
        theme={toasterTheme}
        position="bottom-right"
        closeButton
        richColors
        visibleToasts={5}
        toastOptions={{ duration: 4000 }}
      />
    </TooltipProvider>
  );
}
