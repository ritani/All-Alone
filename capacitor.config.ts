/// <reference types="node" />
import type { CapacitorConfig } from '@capacitor/cli';

/**
 * Native shells for iOS and Android around the same `dist/` web build.
 *
 * - `appId` is a placeholder: the owner sets the real bundle identifier
 *   before signing (see README, "Before publishing").
 * - No `server.url` (the app never loads remote code), no `allowNavigation`,
 *   no cleartext, no mixed content.
 * - Web inspection is off. For local debugging only, run
 *   `CAP_WEBVIEW_DEBUG=1 npx cap sync` and sync again without it before any
 *   release build.
 */
const debug = process.env.CAP_WEBVIEW_DEBUG === '1';

const config: CapacitorConfig = {
  appId: 'com.allalone.game',
  appName: 'All Alone',
  webDir: 'dist',
  backgroundColor: '#0b0c0d',
  loggingBehavior: 'debug',
  server: {
    androidScheme: 'https',
    cleartext: false,
  },
  ios: {
    // The page handles safe areas itself (viewport-fit=cover + env()).
    contentInset: 'never',
    backgroundColor: '#0b0c0d',
    allowsLinkPreview: false,
    preferredContentMode: 'mobile',
    webContentsDebuggingEnabled: debug,
  },
  android: {
    backgroundColor: '#0b0c0d',
    allowMixedContent: false,
    captureInput: false,
    webContentsDebuggingEnabled: debug,
  },
  plugins: {
    // Built into Capacitor 8 (Android): edge-to-edge, honouring viewport-fit=cover,
    // so the page's env(safe-area-inset-*) padding sees the real system bar sizes.
    SystemBars: {
      insetsHandling: 'native',
      initialViewportFitValueHint: 'cover',
      style: 'DARK',
    },
    StatusBar: {
      style: 'DARK',
      backgroundColor: '#0b0c0d',
      overlaysWebView: true,
    },
  },
};

export default config;
