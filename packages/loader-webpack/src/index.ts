import { LoaderContext } from 'webpack';
import { compileToReact } from '@paperclip-lang/wasm';
import { validate } from 'schema-utils';
import { Schema } from 'schema-utils/declarations/ValidationError';

interface LoaderOptions {
  /**
   * Generate TypeScript definitions
   * @default true
   */
  typescript?: boolean;

  /**
   * Emit separate .d.ts file
   * @default false
   */
  emitDeclaration?: boolean;
}

const schema: Schema = {
  type: 'object',
  properties: {
    typescript: {
      type: 'boolean',
    },
    emitDeclaration: {
      type: 'boolean',
    },
  },
  additionalProperties: false,
};

/**
 * Webpack loader for Paperclip files (.pc)
 *
 * Compiles .pc files to React/JSX using WASM
 */
export default function paperclipLoader(
  this: LoaderContext<LoaderOptions>,
  source: string
): void {
  const options: LoaderOptions = this.getOptions();

  // Validate options
  validate(schema, options, {
    name: '@paperclip-lang/webpack-loader',
    baseDataPath: 'options',
  });

  const callback = this.async();
  const filePath = this.resourcePath;

  // Default options
  const typescript = options.typescript !== false;
  const emitDeclaration = options.emitDeclaration === true;

  try {
    // Compile using WASM
    const result = compileToReact(source, filePath, typescript);

    // Emit TypeScript declaration as separate file if requested
    if (emitDeclaration && result.types) {
      const declarationPath = filePath.replace(/\.pc$/, '.d.ts');
      this.emitFile(declarationPath, result.types);
    }

    callback(null, result.code);
  } catch (error) {
    callback(
      error instanceof Error
        ? error
        : new Error(`Failed to compile ${filePath}: ${error}`)
    );
  }
}
