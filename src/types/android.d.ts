declare global {
  interface Window {
    AndroidNative?: {
      shareImageToApp: (imagePath: string, targetApp: string) => void;
      requestStoragePermission: () => void;
    };
  }
}

export {};
