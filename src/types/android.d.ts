declare global {
  interface Window {
    AndroidNative?: {
      shareImageToApp: (imagePath: string, targetApp: string) => void;
      requestStoragePermission: () => void;
      bringToFrontAndFocusSearch: () => void;
      isFloatingWindowEnabled: () => boolean;
      startFloatingWindow: () => void;
      stopFloatingWindow: () => void;
    };
    triggerSearchFocus: () => void;
  }
}

export {};
