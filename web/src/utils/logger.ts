import { invoke } from '@tauri-apps/api/core'

type LogLevel = 'debug' | 'info' | 'warning' | 'error'

/**
 * 记录日志到后端日志系统
 */
export async function log(level: LogLevel, message: string): Promise<void> {
  try {
    await invoke('log_from_frontend', { level, message })
  } catch (error) {
    // 如果后端日志失败，回退到 console
    console.error('记录日志失败:', error)
    console.log(`[${level.toUpperCase()}] ${message}`)
  }
}

export const logger = {
  debug: (message: string) => log('debug', message),
  info: (message: string) => log('info', message),
  warn: (message: string) => log('warning', message),
  error: (message: string) => log('error', message),
}
