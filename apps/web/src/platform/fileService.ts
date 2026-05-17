import { getRuntimeSurface, type RuntimeSurface } from './runtime';

export type FileCommand =
  | 'openProject'
  | 'saveProject'
  | 'saveProjectAs'
  | 'exportV4'
  | 'exportSvg'
  | 'exportPng';

export interface FileCommandResult {
  status: 'handled' | 'not_implemented';
  message: string;
}

export interface FileService {
  surface: RuntimeSurface;
  supportsNativeDialogs: boolean;
  run(command: FileCommand): Promise<FileCommandResult>;
}

const COMMAND_LABELS: Record<FileCommand, string> = {
  openProject: 'Open project',
  saveProject: 'Save project',
  saveProjectAs: 'Save project as',
  exportV4: 'Export v4',
  exportSvg: 'Export SVG',
  exportPng: 'Export PNG',
};

class BrowserFileService implements FileService {
  readonly surface = 'web' as const;
  readonly supportsNativeDialogs = false;

  async run(command: FileCommand): Promise<FileCommandResult> {
    return {
      status: 'not_implemented',
      message: `${COMMAND_LABELS[command]} will use browser file APIs in the persistence phase.`,
    };
  }
}

class TauriFileService implements FileService {
  readonly surface = 'desktop' as const;
  readonly supportsNativeDialogs = true;

  async run(command: FileCommand): Promise<FileCommandResult> {
    return {
      status: 'not_implemented',
      message: `${COMMAND_LABELS[command]} will use Tauri dialogs in the persistence phase.`,
    };
  }
}

export function createFileService(surface: RuntimeSurface): FileService {
  return surface === 'desktop' ? new TauriFileService() : new BrowserFileService();
}

export function getFileService(): FileService {
  return createFileService(getRuntimeSurface());
}
