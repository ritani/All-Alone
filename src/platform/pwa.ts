/**
 * Offline support for the web build: registers dist/sw.js (generated at build
 * time, see vite.config.ts). Production web builds only: never in the dev
 * server, never inside the native apps (they ship their files on the device).
 */
import { detect } from './index';

export function registerServiceWorker(env: { native: boolean } = detect()): boolean {
  if (!import.meta.env.PROD || env.native) return false;
  if (typeof navigator === 'undefined' || !('serviceWorker' in navigator)) return false;
  if (!window.isSecureContext) return false;
  const go = () => {
    navigator.serviceWorker.register('./sw.js', { scope: './' }).catch(() => undefined);
  };
  if (document.readyState === 'complete') go();
  else window.addEventListener('load', go, { once: true });
  return true;
}
