import { Plugin } from 'vite';
import { compileToReact, compileToCss } from '@paperclip-lang/wasm';
import * as path from 'path';

export interface PaperclipPluginOptions {
  /**
   * Generate TypeScript definitions
   * @default true
   */
  typescript?: boolean;

  /**
   * Include CSS in the output
   * @default true
   */
  includeStyles?: boolean;

  /**
   * Filter function to determine which files to process
   * @default (id) => id.endsWith('.pc')
   */
  filter?: (id: string) => boolean;
}

/**
 * Vite plugin for Paperclip files (.pc)
 *
 * Compiles .pc files to React/JSX using WASM
 */
export default function paperclipPlugin(
  options: PaperclipPluginOptions = {}
): Plugin {
  const {
    typescript = true,
    includeStyles = true,
    filter = (id: string) => id.endsWith('.pc'),
  } = options;

  return {
    name: 'paperclip',

    // Handle .pc files
    transform(code, id) {
      if (!filter(id)) {
        return null;
      }

      try {
        // Compile to React
        const result = compileToReact(code, id, typescript);

        let output = result.code;

        // Optionally include styles
        if (includeStyles) {
          try {
            const css = compileToCss(code, id);

            // Inject CSS import
            const cssId = `${id}.css`;
            output = `import '${cssId}';\n${output}`;

            // Return both JS and CSS
            return {
              code: output,
              map: null,
            };
          } catch (cssError) {
            // CSS compilation failed, but continue with JS only
            console.warn(`CSS compilation failed for ${id}:`, cssError);
          }
        }

        return {
          code: output,
          map: null, // TODO: Add source maps
        };
      } catch (error) {
        this.error(`Failed to compile ${id}: ${error}`);
      }
    },

    // Handle hot module replacement
    handleHotUpdate({ file, server }) {
      if (filter(file)) {
        // Invalidate the module and trigger a reload
        const module = server.moduleGraph.getModuleById(file);
        if (module) {
          server.moduleGraph.invalidateModule(module);
          server.ws.send({
            type: 'full-reload',
            path: '*',
          });
        }
      }
    },

    // Configure module resolution
    config() {
      return {
        resolve: {
          extensions: ['.pc'],
        },
      };
    },
  };
}

// Named export for convenience
export { paperclipPlugin };
