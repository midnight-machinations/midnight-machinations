import globals from "globals";
import tseslint from "typescript-eslint";
import pluginReact from "eslint-plugin-react";
import pluginReactHooks from "eslint-plugin-react-hooks";
import { defineConfig } from "eslint/config";
import jsxA11y from "eslint-plugin-jsx-a11y";

export default defineConfig([
  pluginReact.configs.flat.recommended,
  pluginReact.configs.flat['jsx-runtime'],
  pluginReactHooks.configs.flat.recommended,
  jsxA11y.flatConfigs.recommended,
  {
    files: ["**/*.{ts,mts,cts,tsx}"],
    plugins: { tseslint },
    languageOptions: {
      globals: globals.browser,
      parser: tseslint.parser,
      parserOptions: {
        project: './tsconfig.json', // Points to your existing TS config
        tsconfigRootDir: import.meta.dirname, // Tells ESLint where to look
      },
    },
  },
]);
