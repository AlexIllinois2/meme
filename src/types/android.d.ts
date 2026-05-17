declare global {
  interface Window {
    AndroidNative?: {
      shareImageToApp: (imagePath: string, targetApp: string) => void;
      requestStoragePermission: () => void;
      bringToFrontAndFocusSearch: () => void;
      isFloatingWindowEnabled: () => boolean;
      startFloatingWindow: () => void;
      stopFloatingWindow: () => void;
      pickShareApp: () => void;
      syncCustomAppsToPrefs: (packagesJson: string) => void;
      requestUsageStatsPermission: () => void;
      getApplicationName: (packageName: string) => string;
      requestAllPermissionsFromJS: () => void;
      resetPermissionDialogFlag: () => void;
      isAutoSendEnabled: () => boolean;
      setAutoSendEnabled: (enabled: boolean) => void;
      isAccessibilityServiceEnabled: () => boolean;
      openAccessibilitySettings: () => void;
      isSendFlowActive: () => boolean;
      activateSendFlow: () => void;
      deactivateSendFlow: () => void;
    };
    triggerSearchFocus: () => void;
  }
}

export {};
