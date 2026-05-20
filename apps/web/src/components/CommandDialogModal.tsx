import { useEffect, useState } from 'react';
import { CircleAlert, Ruler, X } from 'lucide-react';
import {
  cancelCommandDialog,
  registerCommandDialogHost,
  resolveCommandDialog,
  useCommandDialogStore,
} from '../store/commandDialogStore';
import { Button } from './ui/Button';
import { IconButton } from './ui/IconButton';

export function CommandDialogModal() {
  const dialog = useCommandDialogStore((state) => state.dialog);
  const [draft, setDraft] = useState('');
  const [touched, setTouched] = useState(false);

  useEffect(() => registerCommandDialogHost(), []);

  useEffect(() => {
    if (dialog?.type !== 'number') return;
    setDraft(dialog.initialValue);
    setTouched(false);
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

  if (!dialog) return null;

  const cancelLabel = dialog.cancelLabel ?? 'Cancel';

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
