<template>
  <div class="address-selector">
    <el-cascader
      v-model="selectedCodes"
      :options="cascaderOptions"
      :props="cascaderProps"
      placeholder="选择省/市/区"
      filterable
      clearable
      style="width: 100%"
      @change="handleChange"
    />
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
  findDistrictByName
} from '@/data/china-regions'

interface CascaderOption {
  value: string
  label: string
  children?: CascaderOption[]
}

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

// 选中的编码数组 [provinceCode, cityCode, districtCode]
const selectedCodes = ref<string[]>([])

// 级联选择器配置
const cascaderProps = {
  expandTrigger: 'hover' as const,
  emitPath: true
}

// 构建级联选项
const cascaderOptions = computed<CascaderOption[]>(() => {
  const provinces = getProvinces()
  return provinces.map(province => ({
    value: province.code,
    label: province.name,
    children: getCities(province.code).map(city => ({
      value: city.code,
      label: city.name,
      children: getDistricts(city.code).map(district => ({
        value: district.code,
        label: district.name
      }))
    }))
  }))
})

// 名称映射缓存
const nameMap = computed<Record<string, string>>(() => {
  const map: Record<string, string> = {}
  const provinces = getProvinces()
  provinces.forEach(province => {
    map[province.code] = province.name
    getCities(province.code).forEach(city => {
      map[city.code] = city.name
      getDistricts(city.code).forEach(district => {
        map[district.code] = district.name
      })
    })
  })
  return map
})

// 根据名称初始化编码
function initFromNames(): void {
  if (!props.province) {
    selectedCodes.value = []
    return
  }

  const province = findProvinceByName(props.province)
  if (!province) {
    selectedCodes.value = []
    return
  }

  const codes: string[] = [province.code]

  if (props.city) {
    const city = findCityByName(province.code, props.city)
    if (city) {
      codes.push(city.code)

      if (props.district) {
        const district = findDistrictByName(city.code, props.district)
        if (district) {
          codes.push(district.code)
        }
      }
    }
  }

  selectedCodes.value = codes
}

// 监听外部属性变化
watch([() => props.province, () => props.city, () => props.district], () => {
  initFromNames()
}, { immediate: true })

// 处理选择变化
function handleChange(codes: string[]): void {
  const provinceName = codes[0] ? nameMap.value[codes[0]] || '' : ''
  const cityName = codes[1] ? nameMap.value[codes[1]] || '' : ''
  const districtName = codes[2] ? nameMap.value[codes[2]] || '' : ''

  emit('update:province', provinceName)
  emit('update:city', cityName)
  emit('update:district', districtName)
  emit('change', {
    province: provinceName,
    city: cityName,
    district: districtName
  })
}
</script>

<style scoped>
.address-selector {
  width: 100%;
}
</style>
