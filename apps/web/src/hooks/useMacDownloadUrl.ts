import { getMacDownloadUrl } from '../constants/release';

// Only Apple Silicon builds are currently published; no arch detection needed.
export function useMacDownloadUrl(): string {
  return getMacDownloadUrl();
}
