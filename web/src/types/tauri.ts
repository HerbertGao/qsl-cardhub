import type {
  Card,
  CardFilter,
  CardWithProject,
  PagedCards,
  Project,
  ProjectWithStats,
  Profile,
  TemplateConfig,
  LogEntry,
  AddressEntry
} from './models'

// ========== 项目管理命令 ==========

export interface CreateProjectParams {
  name: string
}

export interface UpdateProjectParams {
  id: string
  name: string
}

export interface DeleteProjectParams {
  id: string
}

// ========== 卡片管理命令 ==========

export interface CreateCardParams {
  project_id: string
  callsign: string
  qty: number
  serial?: string | null
}

export interface ListCardsParams extends CardFilter {}

export interface UpdateCardParams {
  id: string
  callsign?: string
  qty?: number
  status?: string
}

export interface DeleteCardParams {
  id: string
}

export interface DistributeCardParams {
  id: string
  method: string
  address?: string | null
  remarks?: string | null
}

export interface ReturnCardParams {
  id: string
  reason: string
  remarks?: string | null
}

// ========== Profile 配置管理命令 ==========

export interface CreateProfileParams {
  name: string
  printer_name: string
  paper_size: string
}

export interface UpdateProfileParams {
  id: string
  name?: string
  printer_name?: string
  paper_size?: string
}

export interface DeleteProfileParams {
  id: string
}

// ========== 打印命令 ==========

export interface PrintQSLParams {
  profile_id: string
  callsign: string
  name?: string
  address?: string
  qso_data?: Record<string, any>
}

export interface PreviewQSLParams {
  profile_id: string
  callsign: string
  name?: string
  address?: string
  qso_data?: Record<string, any>
}

// ========== QRZ.cn 相关命令 ==========

export interface QRZSaveAndLoginParams {
  username: string
  password: string
}

export interface QRZQueryCallsignParams {
  callsign: string
}

export interface AddressInfo {
  callsign: string
  chinese_address: string
  english_address?: string | null
  source: string
}

// ========== 模板管理命令 ==========

export interface SaveTemplateParams {
  template: TemplateConfig
}

export interface LoadTemplateParams {
  id: string
}

// ========== 日志命令 ==========

export interface GetLogsParams {
  level?: string
  limit?: number
}

// ========== Tauri 命令返回类型映射 ==========

export interface TauriCommands {
  // 项目管理
  create_project_cmd: (params: CreateProjectParams) => Promise<Project>
  list_projects_cmd: () => Promise<ProjectWithStats[]>
  update_project_cmd: (params: UpdateProjectParams) => Promise<Project>
  delete_project_cmd: (params: DeleteProjectParams) => Promise<void>

  // 卡片管理
  create_card_cmd: (params: CreateCardParams) => Promise<Card>
  list_cards_cmd: (params: ListCardsParams) => Promise<PagedCards>
  get_card_cmd: (params: { id: string }) => Promise<CardWithProject>
  get_max_serial_cmd: (params: { project_id: string }) => Promise<number | null>
  update_card_cmd: (params: UpdateCardParams) => Promise<Card>
  delete_card_cmd: (params: DeleteCardParams) => Promise<void>
  distribute_card_cmd: (params: DistributeCardParams) => Promise<Card>
  return_card_cmd: (params: ReturnCardParams) => Promise<Card>

  // Profile 管理
  get_profiles: () => Promise<Profile[]>
  create_profile: (params: CreateProfileParams) => Promise<Profile>
  update_profile: (params: UpdateProfileParams) => Promise<Profile>
  delete_profile: (params: DeleteProfileParams) => Promise<void>
  get_profile: (params: { id: string }) => Promise<Profile>

  // 打印
  print_qsl: (params: PrintQSLParams) => Promise<void>
  preview_qsl: (params: PreviewQSLParams) => Promise<string>

  // QRZ.cn
  qrz_save_and_login: (params: QRZSaveAndLoginParams) => Promise<string>
  qrz_load_credentials: () => Promise<string | null>
  qrz_clear_credentials: () => Promise<void>
  qrz_check_login_status: () => Promise<boolean>
  qrz_test_connection: () => Promise<string>
  qrz_query_callsign: (params: QRZQueryCallsignParams) => Promise<AddressInfo | null>
  qrz_get_address_cache: (params: { card_id: string }) => Promise<AddressEntry[]>

  // 安全/凭据
  check_keyring_available: () => Promise<boolean>

  // 模板
  save_template: (params: SaveTemplateParams) => Promise<void>
  load_template: (params: LoadTemplateParams) => Promise<TemplateConfig>

  // 日志
  get_logs: (params?: GetLogsParams) => Promise<LogEntry[]>
}
