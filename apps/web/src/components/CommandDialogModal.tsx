import { useEffect, useMemo, useState } from 'react';
import { CircleAlert, Download, GitBranch, Ruler, ScanLine, X } from 'lucide-react';
import {
  cancelCommandDialog,
  registerCommandDialogHost,
  resolveCommandDialog,
  useCommandDialogStore,
} from '../store/commandDialogStore';
import { serializeCreasePatternSvg, type CreaseExportOptions } from '../lib/creaseExport';
import { Button } from './ui/Button';
import { IconButton } from './ui/IconButton';
import { SegmentedControl } from './ui/SegmentedControl';
import { Toggle } from './ui/Toggle';

export function CommandDialogModal() {
  const dialog = useCommandDialogStore((state) => state.dialog);
  const [draft, setDraft] = useState('');
  const [exportOptions, setExportOptions] = useState<CreaseExportOptions | null>(null);
  const [touched, setTouched] = useState(false);

  useEffect(() => registerCommandDialogHost(), []);

  useEffect(() => {
    if (dialog?.type !== 'number') return;
    setDraft(dialog.initialValue);
    setTouched(false);
  }, [dialog]);

  useEffect(() => {
    if (dialog?.type !== 'crease-export') return;
    setExportOptions(dialog.initialOptions);
  }, [dialog]);

  useEffect(() => {
    if (!dialog) return undefined;

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        cancelCommandDialog(dialog.id);
      }
    };
    window.addEventListener('keydown', onKeyDown, true);
    return () => window.removeEventListener('keydown', onKeyDown, true);
  }, [dialog]);

  const activeExportOptions =
    dialog?.type === 'crease-export' ? (exportOptions ?? dialog.initialOptions) : null;
  const exportPreviewSrc = useMemo(() => {
    if (dialog?.type !== 'crease-export' || !activeExportOptions) return '';
    const previewSvg = serializeCreasePatternSvg(dialog.project, activeExportOptions);
    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(previewSvg)}`;
  }, [dialog, activeExportOptions]);

  if (!dialog) return null;

  const cancelLabel = dialog.cancelLabel ?? 'Cancel';

  if (dialog.type === 'crease-export') {
    const options = activeExportOptions ?? dialog.initialOptions;
    const confirmLabel = dialog.confirmLabel ?? `Export ${dialog.format.toUpperCase()}`;

    return (
      <div
        role="dialog"
        aria-modal="true"
        aria-label={dialog.title}
        className="simple-modal"
        onMouseDown={() => cancelCommandDialog(dialog.id)}
      >
        <div
          role="document"
          className="simple-modal__document simple-modal__document--export"
          onMouseDown={(event) => event.stopPropagation()}
        >
          <header className="simple-modal__header">
            <span>
              <Download size={15} aria-hidden="true" />
              {dialog.title}
            </span>
            <IconButton size="sm" aria-label={`Close ${dialog.title}`} onClick={() => cancelCommandDialog(dialog.id)}>
              <X size={15} />
            </IconButton>
          </header>
          <form
            className="simple-modal__body export-modal"
            onSubmit={(event) => {
              event.preventDefault();
              resolveCommandDialog(dialog.id, options);
            }}
          >
            <div className="export-modal__preview" aria-label="Export preview">
              <img src={exportPreviewSrc} alt="" />
            </div>
            <div className="export-modal__controls">
              <div className="export-modal__control-group">
                <span className="export-modal__label">View</span>
                <SegmentedControl
                  aria-label="Export view"
                  value={options.viewMode}
                  onChange={(viewMode) =>
                    setExportOptions((current) => ({
                      ...(current ?? options),
                      viewMode,
                    }))
                  }
                  options={[
                    {
                      value: 'mvf',
                      label: 'M/V assignment',
                      icon: <GitBranch size={13} />,
                      title: 'Export mountain, valley, flat, and border fold colors',
                    },
                    {
                      value: 'agrh',
                      label: 'Crease roles',
                      icon: <ScanLine size={13} />,
                      title: 'Export axial, gusset, ridge, hinge, and pseudohinge role colors',
                    },
                  ]}
                />
              </div>
              <div className="export-modal__toggle-row">
                <div className="export-modal__toggle-copy">
                  <span>Include flat / unassigned creases</span>
                </div>
                <Toggle
                  checked={options.includeUnassigned}
                  onChange={(includeUnassigned) => {
                    setExportOptions((current) => ({
                      ...(current ?? options),
                      includeUnassigned,
                    }));
                  }}
                  aria-label="Include flat / unassigned creases"
                />
              </div>
              <div className="export-modal__toggle-row">
                <div className="export-modal__toggle-copy">
                  <span>Show background color</span>
                </div>
                <Toggle
                  checked={options.showBackgroundColor}
                  onChange={(showBackgroundColor) => {
                    setExportOptions((current) => ({
                      ...(current ?? options),
                      showBackgroundColor,
                    }));
                  }}
                  aria-label="Show background color"
                />
              </div>
            </div>
            <footer className="simple-modal__footer">
              <Button size="sm" variant="ghost" onClick={() => cancelCommandDialog(dialog.id)}>
                {cancelLabel}
              </Button>
              <Button size="sm" variant="primary" type="submit">
                {confirmLabel}
              </Button>
            </footer>
          </form>
        </div>
      </div>
    );
  }

  if (dialog.type === 'confirm') {
    const confirmLabel = dialog.confirmLabel ?? 'OK';
    return (
      <div
        role="dialog"
        aria-modal="true"
        aria-label={dialog.title}
        className="simple-modal"
        onMouseDown={() => cancelCommandDialog(dialog.id)}
      >
        <div role="document" className="simple-modal__document" onMouseDown={(event) => event.stopPropagation()}>
          <header className="simple-modal__header">
            <span>
              <CircleAlert size={15} aria-hidden="true" />
              {dialog.title}
            </span>
            <IconButton size="sm" aria-label={`Close ${dialog.title}`} onClick={() => cancelCommandDialog(dialog.id)}>
              <X size={15} />
            </IconButton>
          </header>
          <div className="simple-modal__body">
            <p className="simple-modal__message">{dialog.message}</p>
            <footer className="simple-modal__footer">
              <Button size="sm" variant="ghost" onClick={() => cancelCommandDialog(dialog.id)}>
                {cancelLabel}
              </Button>
              <Button
                size="sm"
                variant={dialog.tone === 'danger' ? 'danger' : 'primary'}
                onClick={() => resolveCommandDialog(dialog.id, true)}
              >
                {confirmLabel}
              </Button>
            </footer>
          </div>
        </div>
      </div>
    );
  }

  const minimum = dialog.minExclusive ?? 0;
  const value = Number.parseFloat(draft);
  const isValid = Number.isFinite(value) && value > minimum;
  const showError = touched && !isValid;
  const confirmLabel = dialog.confirmLabel ?? 'OK';

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={dialog.title}
      className="simple-modal"
      onMouseDown={() => cancelCommandDialog(dialog.id)}
    >
      <div role="document" className="simple-modal__document" onMouseDown={(event) => event.stopPropagation()}>
        <header className="simple-modal__header">
          <span>
            <Ruler size={15} aria-hidden="true" />
            {dialog.title}
          </span>
          <IconButton size="sm" aria-label={`Close ${dialog.title}`} onClick={() => cancelCommandDialog(dialog.id)}>
            <X size={15} />
          </IconButton>
        </header>
        <form
          className="simple-modal__body"
          onSubmit={(event) => {
            event.preventDefault();
            setTouched(true);
            if (!isValid) return;
            resolveCommandDialog(dialog.id, value);
          }}
        >
          <label className="field-row">
            <span>{dialog.label}</span>
            <input
              type="number"
              min={minimum}
              step={dialog.step ?? 0.01}
              value={draft}
              autoFocus
              onChange={(event) => {
                setTouched(true);
                setDraft(event.currentTarget.value);
              }}
            />
          </label>
          {dialog.meta && <div className="simple-modal__meta">{dialog.meta}</div>}
          {showError && (
            <div className="simple-modal__error" role="alert">
              Enter a number greater than {minimum}.
            </div>
          )}
          <footer className="simple-modal__footer">
            <Button size="sm" variant="ghost" onClick={() => cancelCommandDialog(dialog.id)}>
              {cancelLabel}
            </Button>
            <Button size="sm" variant="primary" type="submit" disabled={!isValid}>
              {confirmLabel}
            </Button>
          </footer>
        </form>
      </div>
    </div>
  );
}
