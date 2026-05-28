import * as tauri from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";

export const isTauri = typeof window !== 'undefined' && window.__TAURI__;

export const invoke = isTauri ? tauri.invoke : async () => {
  console.warn("Tauri is not available, running in browser mode");
  return null;
};

export function isAndroidTauri(): boolean {
  return isTauri && /android/i.test(navigator.userAgent);
}

export function toAssetPath(filePath: string | null | undefined): string {
  if (!filePath) return '';
  if (!isTauri) return filePath;
  try {
    return convertFileSrc(filePath);
  } catch (e) {
    console.error('Failed to convert file path:', filePath, e);
    return filePath;
  }
}
