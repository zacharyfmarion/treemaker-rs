import { useEffect, useMemo, useState, type ReactElement } from 'react';
import {
  Check,
  Gauge,
  LayoutDashboard,
  Palette,
  RotateCcw,
  X,
  type LucideIcon,
} from 'lucide-react';
import { requestConfirmation } from '../store/commandDialogStore';
import { useLayoutStore } from '../store/layoutStore';
import {
  MAX_CAMV_ANGLE_TOLERANCE,
  MIN_CAMV_ANGLE_TOLERANCE,
  type SettingsTab,
  useSettingsStore,
} from '../store/settingsStore';
import { useThemeStore } from '../store/themeStore';
import type { TreeMakerTheme } from '../themes';
import { Button } from './ui/Button';
import { IconButton } from './ui/IconButton';

const TABS: Array<{ key: SettingsTab; label: string; icon: LucideIcon }> = [
  { key: 'appearance', label: 'Appearance', icon: Palette },
  { key: 'diagnostics', label: 'Diagnostics', icon: Gauge },
  { key: 'workspace', label: 'Workspace', icon: LayoutDashboard },
];

const TAB_TITLES: Record<SettingsTab, string> = {
  appearance: 'Appearance',
  diagnostics: 'Diagnostics',
  workspace: 'Workspace',
};

function resolveInitialTab(initialTab: SettingsTab | null): SettingsTab {
  return initialTab && TABS.some((tab) => tab.key === initialTab) ? initialTab : 'appearance';
}

function ThemeCard({
  theme,
  selected,
  onSelect,
}: {
  theme: TreeMakerTheme;
  selected: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      className="settings-theme-card"
      data-selected={selected || undefined}
      aria-pressed={selected}
      onClick={onSelect}
    >
      <span className="settings-theme-card__header">
        <span className="settings-theme-card__name">{theme.name}</span>
        {selected && <Check size={14} aria-hidden="true" />}
      </span>
      <span className="settings-theme-card__swatches" aria-hidden="true">
        <span style={{ background: theme.colors['bg.primary'] }} />
        <span style={{ background: theme.colors['bg.secondary'] }} />
        <span style={{ background: theme.colors['accent.primary'] }} />
        <span style={{ background: theme.colors['text.primary'] }} />
        <span style={{ background: theme.colors['status.danger'] }} />
        <span style={{ background: theme.colors['status.success'] }} />
      </span>
    </button>
  );
}

function AppearanceTab() {
  const currentTheme = useThemeStore((state) => state.currentTheme);
  const presetThemes = useThemeStore((state) => state.presetThemes);
  const setTheme = useThemeStore((state) => state.setTheme);

  const themeCategories = useMemo(() => {
    const grouped = presetThemes.reduce<Record<TreeMakerTheme['type'], TreeMakerTheme[]>>(
      (acc, theme) => {
        acc[theme.type].push(theme);
        return acc;
      },
      { dark: [], light: [] }
    );
    return [
      { key: 'dark', label: 'Dark', themes: grouped.dark },
      { key: 'light', label: 'Light', themes: grouped.light },
    ].filter((section) => section.themes.length > 0);
  }, [presetThemes]);

  return (
    <div className="settings-tab">
      {themeCategories.map((section) => (
        <section key={section.key} className="settings-section">
          <h3 className="settings-section__title">{section.label}</h3>
          <div className="settings-theme-grid">
            {section.themes.map((theme) => (
              <ThemeCard
                key={theme.name}
                theme={theme}
                selected={currentTheme.name === theme.name}
                onSelect={() => setTheme(theme)}
              />
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}

function WorkspaceTab() {
  const resetLayout = useLayoutStore((state) => state.resetLayout);

  return (
    <div className="settings-tab">
      <section className="settings-section">
        <h3 className="settings-section__title">Layout</h3>
        <Button
          size="md"
          variant="secondary"
          className="settings-full-width"
          onClick={() => {
            void requestConfirmation({
              title: 'Reset Layout',
              message: 'Restore the default panel layout?',
              confirmLabel: 'Reset',
              tone: 'danger',
            }).then((confirmed) => {
              if (!confirmed) return;
              resetLayout();
            });
          }}
        >
          <RotateCcw size={14} />
          Reset Layout
        </Button>
      </section>
    </div>
  );
}

function DiagnosticsTab() {
  const camvAngleTolerance = useSettingsStore((state) => state.camvAngleTolerance);
  const setCamvAngleTolerance = useSettingsStore((state) => state.setCamvAngleTolerance);
  const resetCamvAngleTolerance = useSettingsStore((state) => state.resetCamvAngleTolerance);

  return (
    <div className="settings-tab">
      <section className="settings-section">
        <h3 className="settings-section__title">CAMV</h3>
        <label className="settings-number-field">
          <span>Angle tolerance</span>
          <input
            aria-label="CAMV angle tolerance"
            type="number"
            min={MIN_CAMV_ANGLE_TOLERANCE}
            max={MAX_CAMV_ANGLE_TOLERANCE}
            step="0.001"
            value={camvAngleTolerance}
            onChange={(event) => {
              const parsed = Number.parseFloat(event.currentTarget.value);
              if (Number.isFinite(parsed)) setCamvAngleTolerance(parsed);
            }}
          />
          <span className="settings-number-field__unit">deg</span>
        </label>
        <Button
          size="md"
          variant="secondary"
          className="settings-full-width"
          onClick={resetCamvAngleTolerance}
        >
          <RotateCcw size={14} />
          Reset CAMV Tolerance
        </Button>
      </section>
    </div>
  );
}

const TAB_COMPONENTS: Record<SettingsTab, () => ReactElement> = {
  appearance: AppearanceTab,
  diagnostics: DiagnosticsTab,
  workspace: WorkspaceTab,
};

function SettingsModalContent({
  initialTab,
  closeSettings,
}: {
  initialTab: SettingsTab;
  closeSettings: () => void;
}) {
  const [activeTab, setActiveTab] = useState<SettingsTab>(initialTab);
  const ActiveTab = TAB_COMPONENTS[activeTab];

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') return;
      event.preventDefault();
      event.stopPropagation();
      closeSettings();
    };
    window.addEventListener('keydown', onKeyDown, true);
    return () => window.removeEventListener('keydown', onKeyDown, true);
  }, [closeSettings]);

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Settings"
      className="settings-modal"
      onMouseDown={closeSettings}
    >
      <div
        role="document"
        className="settings-modal__document"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <aside className="settings-modal__sidebar">
          <div className="settings-modal__title">Settings</div>
          <nav className="settings-modal__tabs" aria-label="Settings sections">
            {TABS.map((tab) => {
              const Icon = tab.icon;
              return (
                <button
                  key={tab.key}
                  type="button"
                  className="settings-modal__tab"
                  data-active={activeTab === tab.key || undefined}
                  onClick={() => setActiveTab(tab.key)}
                >
                  <Icon size={14} aria-hidden="true" />
                  <span>{tab.label}</span>
                </button>
              );
            })}
          </nav>
        </aside>

        <section className="settings-modal__content">
          <header className="settings-modal__header">
            <h2>{TAB_TITLES[activeTab]}</h2>
            <IconButton
              size="sm"
              aria-label="Close settings"
              onClick={closeSettings}
            >
              <X size={15} />
            </IconButton>
          </header>
          <div className="settings-modal__body">
            <ActiveTab />
          </div>
        </section>
      </div>
    </div>
  );
}

export function SettingsModal() {
  const isOpen = useSettingsStore((state) => state.isSettingsOpen);
  const initialTab = useSettingsStore((state) => state.settingsInitialTab);
  const closeSettings = useSettingsStore((state) => state.closeSettings);

  if (!isOpen) return null;

  const resolvedInitialTab = resolveInitialTab(initialTab);
  return (
    <SettingsModalContent
      key={resolvedInitialTab}
      initialTab={resolvedInitialTab}
      closeSettings={closeSettings}
    />
  );
}
