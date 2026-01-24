// 中国省市区数据
// 数据来源: https://github.com/mumuy/data_location

import rawData from './china-regions-raw.json'

export interface RegionData {
  code: string
  name: string
}

// 原始数据类型
const regionMap: Record<string, string> = rawData as Record<string, string>

// 判断是否为省级（后4位为0000）
function isProvince(code: string): boolean {
  return code.endsWith('0000')
}

// 判断是否为市级（后2位为00，前2位不为00）
function isCity(code: string): boolean {
  return code.endsWith('00') && !code.endsWith('0000')
}

// 判断是否为区县级
function isDistrict(code: string): boolean {
  return !code.endsWith('00')
}

// 获取省份编码（前2位 + 0000）
function getProvinceCode(code: string): string {
  return code.substring(0, 2) + '0000'
}

// 获取城市编码（前4位 + 00）
function getCityCode(code: string): string {
  return code.substring(0, 4) + '00'
}

// 缓存
let provincesCache: RegionData[] | null = null
const citiesCache: Record<string, RegionData[]> = {}
const districtsCache: Record<string, RegionData[]> = {}

// 获取省份列表
export function getProvinces(): RegionData[] {
  if (provincesCache) return provincesCache

  provincesCache = Object.entries(regionMap)
    .filter(([code]) => isProvince(code))
    .map(([code, name]) => ({ code, name }))
    .sort((a, b) => a.code.localeCompare(b.code))

  return provincesCache
}

// 获取城市列表（根据省份编码）
export function getCities(provinceCode: string): RegionData[] {
  if (citiesCache[provinceCode]) return citiesCache[provinceCode]

  const provincePrefix = provinceCode.substring(0, 2)

  // 直辖市特殊处理：北京、天津、上海、重庆
  const directCities = ['11', '12', '31', '50']
  if (directCities.includes(provincePrefix)) {
    // 直辖市返回自身作为城市
    citiesCache[provinceCode] = [{
      code: provincePrefix + '0100',
      name: regionMap[provinceCode]
    }]
    return citiesCache[provinceCode]
  }

  const cities = Object.entries(regionMap)
    .filter(([code]) => code.startsWith(provincePrefix) && isCity(code))
    .map(([code, name]) => ({ code, name }))
    .sort((a, b) => a.code.localeCompare(b.code))

  citiesCache[provinceCode] = cities
  return cities
}

// 获取区县列表（根据城市编码）
export function getDistricts(cityCode: string): RegionData[] {
  if (districtsCache[cityCode]) return districtsCache[cityCode]

  const cityPrefix = cityCode.substring(0, 4)
  const provincePrefix = cityCode.substring(0, 2)

  // 直辖市特殊处理
  const directCities = ['11', '12', '31', '50']
  if (directCities.includes(provincePrefix)) {
    // 直辖市的区县直接从省级编码开始查找
    const districts = Object.entries(regionMap)
      .filter(([code]) => code.startsWith(provincePrefix) && isDistrict(code))
      .map(([code, name]) => ({ code, name }))
      .sort((a, b) => a.code.localeCompare(b.code))

    districtsCache[cityCode] = districts
    return districts
  }

  const districts = Object.entries(regionMap)
    .filter(([code]) => code.startsWith(cityPrefix) && isDistrict(code))
    .map(([code, name]) => ({ code, name }))
    .sort((a, b) => a.code.localeCompare(b.code))

  districtsCache[cityCode] = districts
  return districts
}

// 根据名称查找省份
export function findProvinceByName(name: string): RegionData | undefined {
  return getProvinces().find(p => p.name === name || p.name.includes(name) || name.includes(p.name))
}

// 根据名称查找城市
export function findCityByName(provinceCode: string, name: string): RegionData | undefined {
  return getCities(provinceCode).find(c => c.name === name || c.name.includes(name) || name.includes(c.name))
}

// 根据名称查找区县
export function findDistrictByName(cityCode: string, name: string): RegionData | undefined {
  return getDistricts(cityCode).find(d => d.name === name || d.name.includes(name) || name.includes(d.name))
}

// 获取完整地址字符串
export function getFullAddress(provinceCode: string, cityCode: string, districtCode: string): string {
  const province = regionMap[provinceCode] || ''
  const city = regionMap[cityCode] || ''
  const district = regionMap[districtCode] || ''

  // 如果是直辖市，省和市名称相同，只显示一次
  if (province === city) {
    return province + district
  }

  return province + city + district
}

// 根据区县编码获取完整层级信息
export function getRegionInfo(districtCode: string): { province: string; city: string; district: string } | null {
  const district = regionMap[districtCode]
  if (!district) return null

  const provinceCode = getProvinceCode(districtCode)
  const cityCode = getCityCode(districtCode)

  const province = regionMap[provinceCode]
  let city = regionMap[cityCode]

  // 直辖市特殊处理
  const directCities = ['11', '12', '31', '50']
  if (directCities.includes(districtCode.substring(0, 2))) {
    city = province
  }

  return {
    province: province || '',
    city: city || '',
    district
  }
}
