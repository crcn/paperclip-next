import { Plugin } from 'rollup';
import { createFilter, FilterPattern } from '@rollup/pluginutils';
import { compileToReact, compileToCss } from '@paperclip-lang/wasm';

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
   * Files to include (minimatch pattern)
   * @default '**\/*.pc'
   */
  include?: FilterPattern;

  /**
   * Files to exclude (minimatch pattern)
   * @default 'node_modules/**'
   */
  exclude?: FilterPattern;
}

/**
 * Rollup plugin for Paperclip files (.pc)
 *
 * Compiles .pc files to React/JSX using WASM
 */
export default function paperclipPlugin(
  options: PaperclipPluginOptions = {}
): Plugin {
  const {
    typescript = true,
    includeStyles = true,
    include = '**/*.pc',
    exclude = 'node_modules/**',
  } = options;

  const filter = createFilter(include, exclude);

  return {
    name: 'paperclip',

    // Transform .pc files
    async transform(code, id) {
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

            // Emit CSS as a separate chunk
            const cssRefId = this.emitFile({
              type: 'asset',
              name: id.replace(/\.pc$/, '.css'),
              source: css,
            });

            // Import the CSS file
            output = `import './${cssRefId}';\n${output}`;
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
        return null;
      }
    },

    // Add .pc to resolved extensions
    resolveId(source, importer) {
      // Handle .pc imports without extension
      if (importer && !source.endsWith('.pc')) {
        const pcPath = `${source}.pc`;
        return this.resolve(pcPath, importer, { skipSelf: true });
      }
      return null;
    },
  };
}

// Named export for convenience
export { paperclipPlugin };
