// @ts-check
import js from '@eslint/js';
import { defineConfig } from 'eslint/config';
import globals from 'globals';
import tseslint from 'typescript-eslint';

/** Constructs that would let strings become code or markup. */
const unsafeSyntax = [
  {
    selector:
      "AssignmentExpression > MemberExpression.left[property.name=/^(innerHTML|outerHTML)$/]",
    message: 'Do not assign innerHTML/outerHTML. Build DOM through Preact.',
  },
  {
    selector: "MemberExpression[computed=true][property.value=/^(innerHTML|outerHTML)$/]",
    message: 'Do not touch innerHTML/outerHTML.',
  },
  {
    selector: "CallExpression[callee.property.name=/^(insertAdjacentHTML|write|writeln)$/]",
    message: 'insertAdjacentHTML / document.write are banned.',
  },
  {
    selector: "JSXAttribute[name.name='dangerouslySetInnerHTML']",
    message: 'dangerouslySetInnerHTML is banned.',
  },
  {
    selector: "Property[key.name='dangerouslySetInnerHTML']",
    message: 'dangerouslySetInnerHTML is banned.',
  },
  {
    selector: "NewExpression[callee.name='Function'], CallExpression[callee.name='Function']",
    message: 'The Function constructor is banned.',
  },
];

export default defineConfig(
  { ignores: ['dist/', 'coverage/', 'node_modules/', 'ios/', 'android/', 'target/', 'src/core/pkg/', 'src/core/types.ts'] },
  js.configs.recommended,
  tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: { allowDefaultProject: ['eslint.config.js', 'scripts/*.mjs'] },
        tsconfigRootDir: import.meta.dirname,
      },
      globals: { ...globals.browser },
    },
    rules: {
      'no-eval': 'error',
      'no-implied-eval': 'off',
      '@typescript-eslint/no-implied-eval': 'error',
      'no-new-func': 'error',
      'no-script-url': 'error',
      'no-restricted-syntax': ['error', ...unsafeSyntax],
      eqeqeq: ['error', 'always'],
    },
  },
  {
    files: ['eslint.config.js', 'vite.config.ts', 'scripts/**/*.mjs', 'src/**/*.test.ts'],
    languageOptions: { globals: { ...globals.node } },
  },
  { files: ['**/*.js', '**/*.mjs'], extends: [tseslint.configs.disableTypeChecked] },
  {
    // The game core is pure: no UI, no platform, no DOM.
    files: ['src/core/**/*.{ts,tsx}'],
    languageOptions: { globals: {} },
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['**/ui', '**/ui/**', '**/platform', '**/platform/**'],
              message: 'src/core must not import from src/ui or src/platform.',
            },
            {
              group: ['preact', 'preact/**', '@capacitor/*'],
              message: 'src/core is framework- and platform-free.',
            },
          ],
        },
      ],
      'no-restricted-globals': [
        'error',
        'window',
        'document',
        'localStorage',
        'sessionStorage',
        'navigator',
        'fetch',
      ],
    },
  },
);
