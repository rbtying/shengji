module.exports = {
  extends: [
    "standard-with-typescript",
    "plugin:react/recommended",
    "prettier/@typescript-eslint",
    "plugin:prettier/recommended",
  ],
  parserOptions: {
    project: "./tsconfig.json",
    ecmaVersion: 2020,
    ecmaFeatures: {
      jsx: true,
    },
  },
  plugins: ["react"],
  settings: {
    react: {
      version: "detect",
    },
  },
  ignorePatterns: ["**/generated/**"],
  rules: {},
};
