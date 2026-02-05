// ==================== 自动生成的类型（从 Rust 导出） ====================
// 这些类型由 ts-rs 从 Rust 代码自动生成
// 运行 `pnpm generate:types` 重新生成

// 数据库模型
export type { Card } from './generated/Card'
export type { CardMetadata } from './generated/CardMetadata'
export type { CardStatus } from './generated/CardStatus'
export type { CardWithProject } from './generated/CardWithProject'
export type { DistributionInfo } from './generated/DistributionInfo'
export type { ReturnInfo } from './generated/ReturnInfo'
export type { AddressEntry } from './generated/AddressEntry'
export type { Project } from './generated/Project'
export type { ProjectWithStats } from './generated/ProjectWithStats'
export type { PagedCards } from './generated/PagedCards'

// 顺丰模型
export type { SenderInfo } from './generated/SenderInfo'
export type { RecipientInfo } from './generated/RecipientInfo'
export type { SFOrder } from './generated/SFOrder'
export type { SFOrderWithCard } from './generated/SFOrderWithCard'
export type { OrderStatus } from './generated/OrderStatus'
// 兼容别名
export type { OrderStatus as SFOrderStatus } from './generated/OrderStatus'

// 配置模型
export type { Profile } from './generated/Profile'
export type { Platform } from './generated/Platform'
export type { PrinterConfig } from './generated/PrinterConfig'
export type { Template } from './generated/Template'

// ==================== 手动维护的类型（未在 Rust 中定义或参数类型） ====================

// 兼容别名
export type PlatformInfo = import('./generated/Platform').Platform
export type TemplatePathConfig = import('./generated/Template').Template

// 单配置模式的打印机配置
export interface SinglePrinterConfig {
  printer: import('./generated/PrinterConfig').PrinterConfig
  platform: import('./generated/Platform').Platform
}

// 卡片筛选条件（前端参数类型）
export interface CardFilter {
  project_id?: string | null
  callsign?: string | null
  status?: string | null
  page?: number
  page_size?: number
}

// 模板元素
export interface TemplateElement {
  id: string
  type: string
  source?: string
  value?: string
  key?: string
  format?: string
  max_height_mm?: number
  height_mm?: number
  quiet_zone_mm?: number
  human_readable?: boolean
}

// 模板页面配置
export interface TemplatePageConfig {
  dpi: number
  width_mm: number
  height_mm: number
  margin_left_mm: number
  margin_right_mm: number
  margin_top_mm: number
  margin_bottom_mm: number
  border: boolean
  border_thickness_mm?: number
  duplicate_print?: boolean
}

// 模板布局配置
export interface TemplateLayoutConfig {
  align_h: string
  align_v: string
  gap_mm: number
  line_gap_mm: number
}

// 模板元数据
export interface TemplateMetadata {
  name: string
  version: string
  description: string
}

// 模板字体配置
export interface TemplateFonts {
  english: string
  chinese: string
}

// 模板输出配置
export interface TemplateOutputConfig {
  mode: string
  threshold: number
}

// 模板配置（完整）
export interface TemplateConfig {
  page: TemplatePageConfig
  layout: TemplateLayoutConfig
  elements: TemplateElement[]
  metadata: TemplateMetadata
  fonts: TemplateFonts
  output: TemplateOutputConfig
}

// 日志条目
export interface LogEntry {
  timestamp: string
  level: string
  message: string
  target?: string
}

// ==================== 顺丰速运相关参数类型 ====================

// 创建/更新寄件人参数
export interface SenderParams {
  name: string
  phone: string
  mobile?: string | null
  province: string
  city: string
  district: string
  address: string
}

// 创建订单参数
export interface CreateOrderParams {
  sender_id: string
  recipient: import('./generated/RecipientInfo').RecipientInfo
  cargo_name?: string | null
  pay_method?: number | null
  card_id?: string | null
}

// 联系人展示信息
export interface ContactDisplayInfo {
  name: string
  phone: string
  full_address: string
}

// 创建订单响应
export interface CreateOrderResponse {
  order_id: string
  waybill_no_list: string[]
  filter_result?: number | null
  local_order: import('./generated/SFOrder').SFOrder
  sender_info: ContactDisplayInfo
  recipient_info: ContactDisplayInfo
  cargo_name: string
  pay_method: number
  express_type_id: number
  origin_code?: string | null
  dest_code?: string | null
}

// 确认订单响应
export interface ConfirmOrderResponse {
  order_id: string
  waybill_no_list: string[]
  res_status?: number | null
  local_order: import('./generated/SFOrder').SFOrder
}

// 查询订单响应
export interface SearchOrderResponse {
  api_response: {
    success: boolean
    error_code?: string | null
    error_msg?: string | null
    order_id?: string | null
    waybill_no?: string | null
    order_state?: number | null
    order_state_desc?: string | null
  }
}

// 订单列表参数
export interface ListOrdersParams {
  status?: import('./generated/OrderStatus').OrderStatus | null
  card_id?: string | null
  page?: number
  page_size?: number
}

// 订单列表响应
export interface ListOrdersResponse {
  items: import('./generated/SFOrderWithCard').SFOrderWithCard[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

// 地址区域
export interface Region {
  code: string
  name: string
  children?: Region[]
}
