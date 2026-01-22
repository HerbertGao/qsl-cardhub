// 卡片状态枚举
export type CardStatus = 'pending' | 'distributed' | 'returned'

// 分发信息
export interface DistributionInfo {
  method: string
  address?: string | null
  remarks?: string | null
  distributed_at: string
}

// 退卡信息
export interface ReturnInfo {
  method: string
  remarks?: string | null
  returned_at: string
}

// 地址缓存记录
export interface AddressEntry {
  source: string
  chinese_address?: string | null
  english_address?: string | null
  name?: string | null
  mail_method?: string | null
  updated_at?: string | null
  cached_at: string
}

// 卡片元数据
export interface CardMetadata {
  distribution?: DistributionInfo | null
  return?: ReturnInfo | null
  address_cache?: AddressEntry[]
}

// 卡片数据模型
export interface Card {
  id: string
  project_id: string
  creator_id?: string | null
  callsign: string
  qty: number
  status: CardStatus
  metadata?: CardMetadata | null
  created_at: string
  updated_at: string
}

// 带项目信息的卡片
export interface CardWithProject {
  id: string
  project_id: string
  project_name: string
  creator_id?: string | null
  callsign: string
  qty: number
  status: CardStatus
  metadata?: CardMetadata | null
  created_at: string
  updated_at: string
}

// 卡片筛选条件
export interface CardFilter {
  project_id?: string | null
  callsign?: string | null
  status?: string | null
  page?: number
  page_size?: number
}

// 分页卡片结果
export interface PagedCards {
  items: CardWithProject[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

// 项目数据模型
export interface Project {
  id: string
  name: string
  created_at: string
  updated_at: string
}

// 带统计信息的项目
export interface ProjectWithStats {
  id: string
  name: string
  created_at: string
  updated_at: string
  total_cards: number
  pending_cards: number
  distributed_cards: number
  returned_cards: number
}

// 打印机配置
export interface PrinterConfig {
  name: string
  paper_size: string
  orientation: string
}

// 平台信息
export interface PlatformInfo {
  os: string
  arch: string
}

// 模板路径配置
export interface TemplatePathConfig {
  path: string
}

// Profile 配置
export interface Profile {
  id: string
  name: string
  task_name?: string | null
  printer: PrinterConfig
  platform: PlatformInfo
  template: TemplatePathConfig
  template_display_name?: string
  created_at: string
  updated_at: string
}

// 模板字段
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
