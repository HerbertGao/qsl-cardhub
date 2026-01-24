import type { CardWithProject, Project } from './models'

// ========== 对话框通用类型 ==========

export interface DialogEmits {
  (e: 'update:visible', value: boolean): void
  (e: 'confirm', data: any): void
  (e: 'refresh'): void
}

// ========== 卡片相关组件类型 ==========

// 卡片输入对话框
export interface CardInputDialogProps {
  visible: boolean
  projectId: string
}

export interface CardInputFormData {
  callsign: string
  qty: number
  continuous: boolean
}

// 卡片输入对话框确认数据
export interface CardInputConfirmData {
  projectId: string
  callsign: string
  qty: number
  serial: number | null
  continuousMode: boolean
  printAfterSave: boolean
  printerName: string | null
}

// 卡片输入对话框组件实例
export interface CardInputDialogInstance {
  resetForContinuous: () => Promise<void>
}

// 卡片分发对话框
export interface DistributeDialogProps {
  visible: boolean
  card: CardWithProject | null
}

export interface DistributeFormData {
  id: string
  method: string
  address: string | null
  remarks: string | null
}

// 分发对话框确认数据
export interface DistributeConfirmData {
  id: string
  method: string
  address: string | null
  remarks: string | null
}

// 卡片退卡对话框
export interface ReturnDialogProps {
  visible: boolean
  card: CardWithProject | null
}

export interface ReturnFormData {
  id: string
  reason: string
  remarks: string | null
}

// 退卡对话框确认数据
export interface ReturnConfirmData {
  id: string
  method: string
  remarks: string | null
}

// 卡片详情对话框
export interface CardDetailDialogProps {
  visible: boolean
  card: CardWithProject | null
}

// 卡片列表组件
export interface CardListProps {
  projectId?: string | null
}

// ========== 项目相关组件类型 ==========

// 项目对话框
export interface ProjectDialogProps {
  visible: boolean
  project?: Project | null
  mode: 'create' | 'edit'
}

export interface ProjectFormData {
  name: string
}

// 项目列表组件
export interface ProjectListEmits {
  (e: 'select', project: Project): void
  (e: 'refresh'): void
}

// ========== 表单验证规则 ==========

export interface FormRule {
  required?: boolean
  message?: string
  trigger?: string | string[]
  min?: number
  max?: number
  pattern?: RegExp
  validator?: (rule: any, value: any, callback: any) => void
}

export type FormRules = Record<string, FormRule[]>
