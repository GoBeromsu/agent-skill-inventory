#!/usr/bin/env node

import fs from 'node:fs'
import path from 'node:path'
import { execFileSync } from 'node:child_process'

const tsconfigPath = path.join(process.cwd(), 'tsconfig.json')
const tsconfig = JSON.parse(fs.readFileSync(tsconfigPath, 'utf8'))

if (!tsconfig?.compilerOptions?.strict) {
  console.error('[verify:typescript] tsconfig strict 모드가 비활성화되어 있습니다.')
  process.exit(1)
}

try {
  const tscPath = path.join(process.cwd(), 'node_modules', 'typescript', 'bin', 'tsc')
  execFileSync(tscPath, ['-b', '--pretty', 'false'], { stdio: 'inherit' })
  console.log('[verify:typescript] 타입체크 통과')
} catch (error) {
  console.error('[verify:typescript] 타입체크 실패: TypeScript 컴파일 오류를 확인하세요.')
  process.exit(error.status ?? 1)
}
