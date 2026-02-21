#!/usr/bin/env node

import { execFileSync } from 'node:child_process'
import { existsSync } from 'node:fs'

const REQUIRED_TOOLS = [
  { name: 'node', command: ['node', ['-v']], label: 'Node.js', installHint: 'Install via Homebrew (brew install node) or nvm.' },
  { name: 'pnpm', command: ['pnpm', ['-v']], label: 'pnpm', installHint: 'Install via `corepack enable && corepack prepare pnpm@10.30.1 --activate`.' },
  { name: 'rustc', command: ['rustc', ['-V']], label: 'Rust compiler (rustc)', installHint: 'Install via https://rustup.rs/.' },
  { name: 'cargo', command: ['cargo', ['-V']], label: 'Cargo', installHint: 'Install via https://rustup.rs/.' },
]

function runCommand(name, command, args) {
  try {
    const output = execFileSync(command, args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] })
    return output.trim()
  } catch (error) {
    throw new Error(`${name} 실행 실패: ${error.message}`)
  }
}

function checkTool(tool) {
  console.log(`[verify:env] checking ${tool.label}`)
  const version = runCommand(tool.name, tool.command[0], tool.command[1])
  console.log(`[verify:env] ${tool.label} version: ${version}`)
}

function checkWorkspaceDir() {
  const cwd = process.cwd()
  if (!existsSync(cwd)) {
    throw new Error(`[verify:env] 현재 작업 디렉터리가 존재하지 않습니다: ${cwd}`)
  }
}

let failed = false
try {
  checkWorkspaceDir()
  for (const tool of REQUIRED_TOOLS) {
    checkTool(tool)
  }
  console.log('[verify:env] 환경 체크 통과')
} catch (error) {
  failed = true
  const message = error.message
  const maybeTool = REQUIRED_TOOLS.find((tool) => message.includes(tool.name))
  console.error('[verify:env] 실패:', message)
  if (maybeTool) {
    console.error(`[verify:env] 조치: ${maybeTool.installHint}`)
  }
}

process.exit(failed ? 1 : 0)
