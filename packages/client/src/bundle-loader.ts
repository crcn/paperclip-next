/**
 * Bundle Loader - dynamically loads component bundles
 */

export interface BundleLoader {
  load(bundlePath: string): Promise<any>;
}

export function createBundleLoader(): BundleLoader {
  return {
    async load(bundlePath: string) {
      // Use dynamic import with @vite-ignore comment to bypass Vite's static analysis
      return await import(/* @vite-ignore */ bundlePath);
    },
  };
}
