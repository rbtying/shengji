// @ts-check

import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  eslint.configs.recommended,
  tseslint.configs.recommended,
  {
    ignores: ["**/generated/**"]
  },
  {
    rules: {
        "@typescript-eslint/no-unused-vars": [
            "error",
            { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
        ],
        "@typescript-eslint/no-explicit-any": "off",
        // Disable formatting rules that conflict with Prettier
        "quotes": "off",
        "semi": "off",
        "comma-dangle": "off",
        "indent": "off",
        "no-trailing-spaces": "off",
        "object-curly-spacing": "off",
        "array-bracket-spacing": "off",
        "arrow-parens": "off",
        "@typescript-eslint/quotes": "off",
        "@typescript-eslint/semi": "off",
        "@typescript-eslint/comma-dangle": "off",
        "@typescript-eslint/indent": "off",
        "@typescript-eslint/member-delimiter-style": "off",
        "@typescript-eslint/type-annotation-spacing": "off",
    }
  }
);
