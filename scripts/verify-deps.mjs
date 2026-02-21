#!/usr/bin/env node

import { createRequire } from 'node:module'
import path from 'node:path'

const cwd = process.cwd()
const require = createRequire(path.join(cwd, 'package.json'))

const REQUIRED_PACKAGES = [
  { name: 'react-refresh', specifier: 'react-refresh', install: 'pnpm add -D react-refresh' },
  { name: '@vitejs/plugin-react', specifier: '@vitejs/plugin-react', install: 'pnpm add -D @vitejs/plugin-react' },
  { name: '@types/react', specifier: '@types/react/package.json', install: 'pnpm add -D @types/react' },
  { name: '@types/react-dom', specifier: '@types/react-dom/package.json', install: 'pnpm add -D @types/react-dom' },
  { name: 'react/jsx-runtime', specifier: 'react/jsx-runtime', install: 'pnpm add react' },
]

let hasFailure = false

for (const dependency of REQUIRED_PACKAGES) {
  try {
    const resolved = require.resolve(dependency.specifier)
    console.log(`[verify:deps] ${dependency.name} -> ${resolved}`)
  } catch (error) {
    hasFailure = true
    console.error(`[verify:deps] missing: ${dependency.name}`)
    console.error(`[verify:deps] install: ${dependency.install}`)
  }
}

if (hasFailure) {
  console.error('[verify:deps] 의존성 확인 실패')
  process.exit(1)
}

console.log('[verify:deps] 핵심 의존성 확인 완료')
