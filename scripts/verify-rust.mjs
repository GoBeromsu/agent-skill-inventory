#!/usr/bin/env node

import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'

const projectRoot = process.cwd()
const distPath = path.join(projectRoot, 'dist')
const placeholderPath = path.join(distPath, 'index.html')
let createdPlaceholder = false

if (!fs.existsSync(distPath)) {
  fs.mkdirSync(distPath, { recursive: true })
  fs.writeFileSync(
    placeholderPath,
    '<!doctype html><html><body><div id="root">placeholder</div></body></html>',
    'utf8',
  )
  createdPlaceholder = true
}

const manifestPath = path.join(projectRoot, 'src-tauri', 'Cargo.toml')

try {
  execFileSync('cargo', ['check', '--manifest-path', manifestPath], { stdio: 'inherit' })
  console.log('[verify:rust] Rust 프로젝트 체크 통과')
} catch (error) {
  console.error('[verify:rust] Rust 체크 실패: cargo check 오류를 확인하세요.')
  process.exit(error.status ?? 1)
} finally {
  if (createdPlaceholder && fs.existsSync(placeholderPath)) {
    fs.rmSync(placeholderPath)
    fs.rmdirSync(distPath)
  }
}
