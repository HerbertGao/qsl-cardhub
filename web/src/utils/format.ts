// 格式化工具函数

/**
 * 格式化序列号显示（3位数，前导零）
 * @param serial 序列号（数字或字符串）
 * @returns 格式化后的序列号字符串，如 "001"、"012"、"123"
 */
export function formatSerial(serial: number | string | null | undefined): string {
  if (serial === null || serial === undefined || serial === '') {
    return ''
  }
  return String(serial).padStart(3, '0')
}
