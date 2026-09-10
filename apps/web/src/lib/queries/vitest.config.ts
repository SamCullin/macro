import { fileURLToPath } from 'node:url';
import solidPlugin from 'vite-plugin-solid';
import tsconfigPaths from 'vite-tsconfig-paths';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [
    tsconfigPaths({
      root: '../../../',
    }),
    solidPlugin(),
  ],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: [
      fileURLToPath(new URL('../core/vitest.setup.ts', import.meta.url)),
    ],
    deps: {
      optimizer: {
        web: {
          include: ['solid-js', '@tanstack/solid-query', 'zod'],
        },
      },
    },
  },
} as any);
