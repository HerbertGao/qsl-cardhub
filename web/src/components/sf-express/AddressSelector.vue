<template>
  <div class="address-selector">
    <el-row :gutter="12">
      <el-col :span="8">
        <el-select
          v-model="selectedProvinceCode"
          placeholder="选择省份"
          filterable
          clearable
          style="width: 100%"
          @change="handleProvinceChange"
        >
          <el-option
            v-for="province in provinceList"
            :key="province.code"
            :label="province.name"
            :value="province.code"
          />
        </el-select>
      </el-col>
      <el-col :span="8">
        <el-select
          v-model="selectedCityCode"
          placeholder="选择城市"
          filterable
          clearable
          :disabled="!selectedProvinceCode"
          style="width: 100%"
          @change="handleCityChange"
        >
          <el-option
            v-for="city in cityList"
            :key="city.code"
            :label="city.name"
            :value="city.code"
          />
        </el-select>
      </el-col>
      <el-col :span="8">
        <el-select
          v-model="selectedDistrictCode"
          placeholder="选择区县"
          filterable
          clearable
          allow-create
          :disabled="!selectedCityCode"
          style="width: 100%"
          @change="handleDistrictChange"
        >
          <el-option
            v-for="district in districtList"
            :key="district.code"
            :label="district.name"
            :value="district.code"
          />
        </el-select>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  getProvinces,
  getCities,
  getDistricts,
  findProvinceByName,
  findCityByName,
  findDistrictByName,
  type RegionData
} from '@/data/china-regions'

interface Props {
  province?: string
  city?: string
  district?: string
}

interface Emits {
  (e: 'update:province', value: string): void
  (e: 'update:city', value: string): void
  (e: 'update:district', value: string): void
  (e: 'change', value: { province: string; city: string; district: string }): void
}

const props = withDefaults(defineProps<Props>(), {
  province: '',
  city: '',
  district: ''
})

const emit = defineEmits<Emits>()

// 内部状态（使用编码）
const selectedProvinceCode = ref('')
const selectedCityCode = ref('')
const selectedDistrictCode = ref('')

// 省份列表
const provinceList = computed<RegionData[]>(() => getProvinces())

// 城市列表
const cityList = computed<RegionData[]>(() => {
  if (!selectedProvinceCode.value) return []
  return getCities(selectedProvinceCode.value)
})

// 区县列表
const districtList = computed<RegionData[]>(() => {
  if (!selectedCityCode.value) return []
  return getDistricts(selectedCityCode.value)
})

// 根据名称初始化编码
function initFromNames(): void {
  if (props.province) {
    const province = findProvinceByName(props.province)
    if (province) {
      selectedProvinceCode.value = province.code

      if (props.city) {
        const city = findCityByName(province.code, props.city)
        if (city) {
          selectedCityCode.value = city.code

          if (props.district) {
            const district = findDistrictByName(city.code, props.district)
            if (district) {
              selectedDistrictCode.value = district.code
            }
          }
        }
      }
    }
  }
}

// 监听外部属性变化
watch([() => props.province, () => props.city, () => props.district], () => {
  initFromNames()
}, { immediate: true })

// 获取名称（根据编码或列表）
function getProvinceName(code: string): string {
  const province = provinceList.value.find(p => p.code === code)
  return province?.name || ''
}

function getCityName(code: string): string {
  const city = cityList.value.find(c => c.code === code)
  return city?.name || ''
}

function getDistrictName(code: string): string {
  // 支持手动输入的区县名称（非编码格式）
  if (code && !code.match(/^\d{6}$/)) {
    return code
  }
  const district = districtList.value.find(d => d.code === code)
  return district?.name || code
}

// 处理省份变化
function handleProvinceChange(code: string): void {
  selectedProvinceCode.value = code
  selectedCityCode.value = ''
  selectedDistrictCode.value = ''

  const provinceName = getProvinceName(code)
  emit('update:province', provinceName)
  emit('update:city', '')
  emit('update:district', '')
  emitChange()
}

// 处理城市变化
function handleCityChange(code: string): void {
  selectedCityCode.value = code
  selectedDistrictCode.value = ''

  const cityName = getCityName(code)
  emit('update:city', cityName)
  emit('update:district', '')
  emitChange()
}

// 处理区县变化
function handleDistrictChange(code: string): void {
  selectedDistrictCode.value = code

  const districtName = getDistrictName(code)
  emit('update:district', districtName)
  emitChange()
}

// 触发变化事件
function emitChange(): void {
  emit('change', {
    province: getProvinceName(selectedProvinceCode.value),
    city: getCityName(selectedCityCode.value),
    district: getDistrictName(selectedDistrictCode.value)
  })
}
</script>

<style scoped>
.address-selector {
  width: 100%;
}
</style>
