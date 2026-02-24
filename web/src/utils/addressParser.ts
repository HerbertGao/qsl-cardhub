import { getProvinces, getCities, getDistricts } from '@/data/china-regions'
import type { RegionData } from '@/data/china-regions'

export interface ParsedAddress {
  name: string
  phone: string
  province: string
  city: string
  district: string
  address: string
}

/**
 * 从自由文本中解析姓名、手机号、省市区、详细地址
 */
export function parseAddress(raw: string): ParsedAddress {
  const result: ParsedAddress = {
    name: '',
    phone: '',
    province: '',
    city: '',
    district: '',
    address: ''
  }

  if (!raw.trim()) return result

  let text = raw.trim()

  // 1. 提取手机号：1[3-9] 开头的 11 位数字，允许中间有横杠/空格
  const phoneRegex = /1[3-9][\d\s-]{9,13}/g
  const phoneMatch = text.match(phoneRegex)
  if (phoneMatch) {
    const cleaned = phoneMatch[0].replace(/[\s-]/g, '')
    if (/^1[3-9]\d{9}$/.test(cleaned)) {
      result.phone = cleaned
      text = text.replace(phoneMatch[0], ' ')
    }
  }

  // 2. 清理分隔符，统一为空格
  text = text.replace(/[,，;；\t\n\r]+/g, ' ').replace(/\s+/g, ' ').trim()

  // 3. 匹配省份
  const provinces = getProvinces()
  const provinceMatch = findBestRegionMatch(text, provinces, ['省', '市', '自治区', '特别行政区'])
  if (provinceMatch) {
    result.province = provinceMatch.region.name
    text = removeMatchedPart(text, provinceMatch.matchedText)

    // 4. 匹配城市
    const cities = getCities(provinceMatch.region.code)
    const cityMatch = findBestRegionMatch(text, cities, ['市', '地区', '盟', '自治州', '州'])
    if (cityMatch) {
      result.city = cityMatch.region.name
      text = removeMatchedPart(text, cityMatch.matchedText)

      // 5. 匹配区县
      const districts = getDistricts(cityMatch.region.code)
      const districtMatch = findBestRegionMatch(text, districts, ['区', '县', '市', '旗'])
      if (districtMatch) {
        result.district = districtMatch.region.name
        text = removeMatchedPart(text, districtMatch.matchedText)
      }
    } else {
      // 直辖市：省=市，直接匹配区县
      const directCityCodes = ['11', '12', '31', '50']
      const provincePrefix = provinceMatch.region.code.substring(0, 2)
      if (directCityCodes.includes(provincePrefix)) {
        result.city = provinceMatch.region.name
        const cityCode = provincePrefix + '0100'
        const districts = getDistricts(cityCode)
        const districtMatch = findBestRegionMatch(text, districts, ['区', '县', '市', '旗'])
        if (districtMatch) {
          result.district = districtMatch.region.name
          text = removeMatchedPart(text, districtMatch.matchedText)
        }
      }
    }
  }

  // 6. 剩余文本拆分为姓名和详细地址
  text = text.replace(/\s+/g, ' ').trim()
  if (text) {
    const parts = text.split(/\s+/)
    if (parts.length === 1) {
      // 单段文本：短的当姓名（≤5字符且非数字开头），否则当地址
      if (parts[0].length <= 5 && !/^\d/.test(parts[0])) {
        result.name = parts[0]
      } else {
        result.address = parts[0]
      }
    } else {
      // 多段文本：找出最短且≤5字符的当姓名，其余拼为地址
      let nameIndex = -1
      let minLen = Infinity
      for (let i = 0; i < parts.length; i++) {
        if (parts[i].length <= 5 && !/^\d/.test(parts[i]) && parts[i].length < minLen) {
          minLen = parts[i].length
          nameIndex = i
        }
      }
      if (nameIndex >= 0) {
        result.name = parts[nameIndex]
        parts.splice(nameIndex, 1)
      }
      result.address = parts.join('')
    }
  }

  return result
}

interface RegionMatchResult {
  region: RegionData
  matchedText: string
}

/**
 * 在文本中查找最佳匹配的地区名称
 * 支持带/不带行政后缀的匹配
 */
function findBestRegionMatch(
  text: string,
  regions: RegionData[],
  suffixes: string[]
): RegionMatchResult | null {
  let bestMatch: RegionMatchResult | null = null
  let bestLength = 0

  for (const region of regions) {
    // 完整名称匹配（优先级最高）
    if (text.includes(region.name) && region.name.length > bestLength) {
      bestMatch = { region, matchedText: region.name }
      bestLength = region.name.length
    }

    // 去掉后缀的简称匹配
    for (const suffix of suffixes) {
      if (region.name.endsWith(suffix)) {
        const shortName = region.name.slice(0, -suffix.length)
        if (shortName.length >= 2 && text.includes(shortName) && shortName.length > bestLength) {
          bestMatch = { region, matchedText: shortName }
          bestLength = shortName.length
        }
      }
    }
  }

  return bestMatch
}

function removeMatchedPart(text: string, matched: string): string {
  const idx = text.indexOf(matched)
  if (idx === -1) return text
  return (text.slice(0, idx) + ' ' + text.slice(idx + matched.length)).replace(/\s+/g, ' ').trim()
}
