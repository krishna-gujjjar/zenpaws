import { defineConfig } from "oxlint";
import core from "ultracite/oxlint/core";
import jsPlugins from "ultracite/oxlint/js-plugins";
import react from "ultracite/oxlint/react";
import tanstack from "ultracite/oxlint/tanstack";

const selectedJsPluginNames = new Set(["sonarjs", "react-doctor", "github"]);
const selectedJsPluginRulePrefixes = new Set([
  "sonarjs",
  "react-doctor",
  "github",
]);

const selectedJsPlugins = {
  ...jsPlugins,
  jsPlugins: jsPlugins.jsPlugins?.filter((plugin) =>
    selectedJsPluginNames.has((plugin as { name: string }).name)
  ),
  overrides: jsPlugins.overrides?.map((override) => ({
    ...override,
    rules: Object.fromEntries(
      Object.entries(override.rules ?? {}).filter(([ruleName]) =>
        selectedJsPluginRulePrefixes.has(ruleName.split("/")[0] ?? ruleName)
      )
    ),
  })),
  rules: Object.fromEntries(
    Object.entries(jsPlugins.rules ?? {}).filter(([ruleName]) =>
      selectedJsPluginRulePrefixes.has(ruleName.split("/")[0] ?? ruleName)
    )
  ),
};

export default defineConfig({
  extends: [core, jsPlugins, react, tanstack, selectedJsPlugins],
  ignorePatterns: core.ignorePatterns,
  rules: {
    "eslint/func-style": "off",
    "eslint/no-empty-function": "off",
    "eslint/no-nested-ternary": "off",
    "eslint/no-shadow": "off",
    "eslint/no-use-before-define": "off",
    "jsx-a11y/no-noninteractive-element-interactions": "off",
    "jsx-a11y/prefer-tag-over-role": "off",
    "promise/prefer-await-to-then": "off",
    "react-hooks/exhaustive-deps": "off",
    "react/hook-use-state": "off",
    "react/react-compiler": "off",
  },
});