#### 订单结果查询接口-速运类API

###### EXP_RECE_SEARCH_ORDER_RESP

-------------

###### 1. 功能描述

- 因Internet环境下，网络不是绝对可靠，用户系统下订单到顺丰后，不一定可以收到顺丰系统返回的数据，此接口用于在未收到返回数据时，查询订单创建接口客户订单当前的处理情况。

##### 2. 接口定义

###### 2.1. 公共参数

| 名称     | 值                                             |
|--------|-----------------------------------------------|
| 接口服务代码 | EXP_RECE_SEARCH_ORDER_RESP                    |
| 生产环境地址 | https://bspgw.sf-express.com/std/service      |
| 香港生产环境 | https://sfapi-hk.sf-express.com/std/service   |
| 沙箱环境地址 | https://sfapi-sbox.sf-express.com/std/service |
| 批量交易   | 不支持                                           |
| 接口类型   | 接入                                            |
| 报文类型   | JSON/XML                                      |

**关注点：**[详见开发规范](https://open.sf-express.com/developSupport/976720)
①通讯双方采用HTTP作为通讯协议；
②提交方式为POST方式，请求头须添加"Content-type","application/x-www-form-urlencoded” 字符集编码统一使用UTF-8；
③参数需要通过http URL编码传送
④业务数据统一以字符串格式放在msgData字段中传送

###### <a id="commonReqParam">2.2. 公共请求参数

| # | 参数列表        | 类型          | 是否必传 | 含义                                                                                                              |
|:-:|:------------|:------------|:----:|:----------------------------------------------------------------------------------------------------------------|
| 1 | partnerID   | String(64)  |  是   | 合作伙伴编码/顾客编码（[获取指引](https://open.sf-express.com/developSupport/195960)）                                          |
| 2 | requestID   | String(40)  |  是   | 请求唯一号UUID                                                                                                       |
| 3 | serviceCode | String(50)  |  是   | 接口服务代码（EXP_RECE_SEARCH_ORDER_RESP）                                                                              |
| 4 | timestamp   | long        |  是   | 调用接口时间戳                                                                                                         |
| 5 | msgDigest   | String(128) |  条件  | 数字签名,使用数字签名方式认证时必填 <br/>签名方法参考：[数字签名认证说明](https://open.sf-express.com/developSupport/976720?authId=1)     |
| 6 | accessToken | String      |  条件  | 访问令牌，使用OAuth2方式认证时必填 <br/>获取方法参考：[OAuth2认证说明](https://open.sf-express.com/developSupport/976720?authId=0) |
| 7 | msgData     | String      |  是   | 业务数据报文                                                                                                          |

###### 2.3. 请求参数<msgData> /OrderSearchReqDto

| # | 属性名           | 类型(约束)     | 必填 | 默认值 | 描述                                                                       |                                                              |
|---|---------------|------------|----|-----|--------------------------------------------------------------------------|--------------------------------------------------------------|
| 1 | orderId       | String(64) | 是  |     | 客户订单号                                                                    |
| 2 | searchType    | String(10) | 否  |     | 查询类型：1正向单  2退货单                                                          |
| 3 | language      | String(10) | 否  |     | 响应报文的语言， 缺省值为zh-CN，目前支持以下值zh-CN 表示中文简体， zh-TW或zh-HK或 zh-MO表示中文繁体， en表示英文 |
| 4 | mainWaybillNo | String(15) | 条件 |     | 顺丰下单接口返回的15或12位运单号,母单号 如：SF10116351372291                                |

###### <a id="commonRespParam">2.4. 公共响应参数 

| # | 属性名       | 类型(约束) | 必填 | 默认值 | 描述                   |
|---|-----------|--------|----|-----|----------------------|
| 1 | success   | String | 是  |     | true 请求成功，false 请求失败 |
| 2 | errorCode | String | 是  |     | 错误编码，S0000成功         |
| 3 | errorMsg  | String | 是  |     | 错误描述                 |
| 4 | msgData   | String | 是  |     | 返回的详细数据              |

###### <a id="respParam">2.5. 响应参数<msgData> /OrderSearchRespDto

| # | 属性名                 | 类型(约束)     | 必填 | 默认值 | 描述                         |
|---|---------------------|------------|----|-----|----------------------------|
| 1 | orderId             | String(64) | 是  |     | 客户订单号                      |
| 2 | origincode          | String(30) | 否  | 1   | 原寄地区域代码，可用 于顺丰电子运单标签打印     |
| 3 | destcode            | String(30) | 否  |     | 目的地区域代码，可用 于顺丰电子运单标签打印     |
| 4 | filterResult        | String(30) | 否  |     | 筛单结果： 1：人工确认 2：可收派 3：不可以收派 |
| 5 | returnExtraInfoList | List       | 否  |     | 返回信息扩展属性                   |
| 6 | waybillNoInfoList   | List       | 否  |     | 顺丰运单号                      |
| 7 | routeLabelInfo      | List       | 否  |     | 路由标签数据                     |

###### 2.5.1  元素<响应> returnExtraInfoList/List

| # | 属性名      | 类型(约束)     | 必填 | 描述    |
|---|----------|------------|----|-------|
| 1 | attrName | String(30) | 是  | 扩展属性名 |
| 2 | attrVal  | String(30) | 是  | 扩展属性值 |

###### 2.5.2  元素<响应> waybillNoInfoList/List

| # | 属性名         | 类型(约束)     | 必填 | 描述                       |
|---|-------------|------------|----|--------------------------|
| 1 | waybillType | String(30) | 是  | 运单号类型 1：母单 2 :子单 3 : 签回单 |
| 2 | waybillNo   | String(30) | 是  | 运单号                      |

###### 2.5.3  元素<响应> routeLabelInfo/List

| # | 属性名            | 类型(约束)                  | 必填 | 描述                                                                                                                |
|---|----------------|-------------------------|----|-------------------------------------------------------------------------------------------------------------------|
| 1 | code           | String(30)              | 是  | 0000（接口参数异常） 0010（其它异常） 0001（xml解析异常） 0002（字段校验异常） 0003（票数节点超出最大值， 批量请求最大票数为100票） 0004（RLS获取路由标签的必要 字段为空） 1000 成功 |
| 2 | message        | String(30)              | 否  | code为0XXX时的错误消息                                                                                                   
| 3 | routeLabelData | RouteLabelRespDetailDto | 否  | 路由标签响应详情                                                                                                          |

###### 2.5.4  元素<响应> routeLabelInfo/List/routeLabelData/RouteLabelRespDetailDto

| #  | 属性名                 | 类型(约束)       | 必填 | 默认值 | 描述                                                                                                                                                                                   |
|----|---------------------|--------------|----|-----|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1  | waybillNo           | String(30)   | 否  |     | 运单号                                                                                                                                                                                  
| 2  | sourceTransferCode  | String(60)   | 否  |     | 原寄地中转场                                                                                                                                                                               
| 3  | sourceCityCode      | String(60)   | 否  |     | 原寄地城市代码                                                                                                                                                                              
| 4  | sourceDeptCode      | String(60)   | 否  |     | 原寄地网点代码                                                                                                                                                                              
| 5  | sourceTeamCode      | String(60)   | 否  |     | 原寄地单元区域                                                                                                                                                                              
| 6  | destCityCode        | String(60)   | 否  |     | 目的地城市代码,eg:755                                                                                                                                                                       
| 7  | destDeptCode        | String(60)   | 否  |     | 目的地网点代码,eg:755AQ                                                                                                                                                                     
| 8  | destDeptCodeMapping | String(60)   | 否  |     | 目的地网点代码映射码                                                                                                                                                                           
| 9  | destTeamCode        | String(60)   | 否  |     | 目的地单元区域,eg:001                                                                                                                                                                       
| 10 | destTeamCodeMapping | String(60)   | 否  |     | 目的地单元区域映射码                                                                                                                                                                           
| 11 | destTransferCode    | String(60)   | 否  |     | 目的地中转场                                                                                                                                                                               
| 12 | destRouteLabel      | String(200)  | 否  |     | 打单时的路由标签信息如果是大网的路由标签，这里的值是目的地网点代码，如果是同城配的路由标签，这里的值是根据同城配的设置映射出来的值，不同的配置结果会不一样，不能根据-符号切分（如：上海同城配，可能是：集散点-目的地网点-接驳点，也有可能是目的地网点代码-集散点-接驳点）                                              
| 13 | twoDimensionCode    | String(600)  | 否  |     | 二维码根据规则生成字符串信息,格式为MMM={'k1':'（目的地中转场代码）','k2':'（目的地原始网点代码）','k3':'（目的地单元区域）','k4':'（附件通过三维码（express_type_code、 limit_type_code、 cargo_type_code）映射时效类型）','k5':'（运单号）'，'k6':'（AB标识）'} 
| 14 | proCode             | String(30)   | 否  |     | 时效类型:值为二维码中的K4                                                                                                                                                                       
| 15 | printIcon           | String(100)  | 否  |     | 打印图标根据托寄物判断需要打印的图标(重货,蟹类,生鲜,易碎，Z标)?返回值有8位，每一位只有0和1两种，0表示按运单默认的规则，1表示显示。后面两位默认0备用。顺序如下：重货,蟹类,生鲜,易碎,医药类,Z标,0,0如：00000000表示不需要打印重货，蟹类，生鲜，易碎,医药,Z标,备用,备用                                 
| 16 | abFlag              | String(30)   | 否  |     | AB标                                                                                                                                                                                  
| 17 | errMsg              | String(1000) | 否  |     | 查询出现异常时返回信息。返回代码：0 系统异常1 未找到运单                                                                                                                                                       
| 18 | destPortCode        | String（100）  | 否  |     | 目的地口岸代码                                                                                                                                                                              
| 19 | destCountry         | String(50)   | 否  |     | 目的国别(国别代码如：JP)                                                                                                                                                                       
| 20 | destPostCode        | String(100   | 否  |     | 目的地邮编                                                                                                                                                                                
| 21 | goodsValueTotal     | String(30)   | 否  |     | 总价值(保留两位小数，数字类型，可补位)                                                                                                                                                                 
| 22 | currencySymbol      | String（30）   | 否  |     | 币种                                                                                                                                                                                   
| 23 | goodsNumber         | String(20)   | 否  |     | 件数                                                                                                                                                                                   
| 24 | twoDimensionCode    | String(600)  | 否  |     | 二维码根据规则生成字符串信息,格式为MMM={'k1':'（目的地中转场代码）','k2':'（目的地原始网点代码）','k3':'（目的地单元区域）','k4':'（附件通过三维码（express_type_code、 limit_type_code、 cargo_type_code）映射时效类型）','k5':'（运单号）'，'k6':'（AB标识）'} 
| 25 | proCode             | String(30)   | 否  |     | 时效类型:值为二维码中的K4                                                                                                                                                                       
| 26 | printIcon           | String(100)  | 否  |     | 打印图标根据托寄物判断需要打印的图标(重货,蟹类,生鲜,易碎，Z标)?返回值有8位，每一位只有0和1两种，0表示按运单默认的规则，1表示显示。后面两位默认0备用。顺序如下：重货,蟹类,生鲜,易碎,医药类,Z标,0,0如：00000000表示不需要打印重货，蟹类，生鲜，易碎,医药,Z标,备用,备用                                 
| 27 | abFlag              | String(30)   | 否  |     | AB标                                                                                                                                                                                  
| 28 | errMsg              | String(1000) | 否  |     | 查询出现异常时返回信息。返回代码：0 系统异常1 未找到运单                                                                                                                                                       
| 29 | destPortCode        | String（100）  | 否  |     | 目的地口岸代码                                                                                                                                                                              
| 30 | destCountry         | String(50)   | 否  |     | 目的国别(国别代码如：JP)                                                                                                                                                                       
| 31 | destPostCode        | String(100   | 否  |     | 目的地邮编                                                                                                                                                                                
| 32 | goodsValueTotal     | String(30)   | 否  |     | 总价值(保留两位小数，数字类型，可补位)                                                                                                                                                                 
| 33 | currencySymbol      | String（30）   | 否  |     | 币种                                                                                                                                                                                   
| 34 | goodsNumber         | String(20)   | 否  |     | 件数                                                                                                                                                                                   

###### <a id="requestJsonPost">2.6. 请求示例\应用场景(JSON)示例 

请求报文:

```json
{
  "searchType": "1",
  "orderId": "TE201407020016",
  "language": "zh-cn"
}
```

###### <a id="responseJsonPost">2.7. 返回示例\应用场景(JSON)示例 

响应报文:

- 成功响应:

```json
{
  "errorCode": "S0000",
  "msgData": {
    "destcode": "020",
    "filterResult": "2",
    "orderId": "2020051114001",
    "origincode": "769",
    "routeLabelInfo": [
      {
        "code": "1000",
        "message": "SF7444400034485:",
        "routeLabelData": {
          "abFlag": "",
          "cargoTypeCode": "C201",
          "checkCode": "5f5f8a2d",
          "codingMapping": "C50",
          "codingMappingOut": "1",
          "currencySymbol": "",
          "cusBatch": "",
          "destCityCode": "020",
          "destCountry": "",
          "destDeptCode": "020AA",
          "destDeptCodeMapping": "020W",
          "destGisDeptCode": "020AA",
          "destPortCode": "",
          "destPostCode": "",
          "destRouteLabel": "020AA",
          "destTeamCode": "",
          "destTeamCodeMapping": "",
          "destTransferCode": "020",
          "errMsg": "",
          "expressTypeCode": "B1",
          "fbaIcon": "",
          "fileIcon": "",
          "goodsNumber": "",
          "goodsValueTotal": "",
          "icsmIcon": "",
          "limitTypeCode": "T4",
          "printFlag": "000000000",
          "printIcon": "00100000",
          "proCode": "T4",
          "proIcon": "",
          "proName": "顺丰标快",
          "sourceCityCode": "769",
          "sourceDeptCode": "769",
          "sourceTeamCode": "",
          "sourceTransferCode": "",
          "twoDimensionCode": "MMM={'k1':'020','k2':'020AA','k3':'','k4':'T4','k5':'SF7444400034485','k6':'','k7':'5f5f8a2d'}",
          "waybillNo": "SF7444400034485",
          "xbFlag": "0"
        }
      },
      {
        "code": "1000",
        "message": "SF7444600201958:",
        "routeLabelData": {
          "abFlag": "",
          "cargoTypeCode": "C201",
          "checkCode": "ae1f2320",
          "codingMapping": "K01",
          "codingMappingOut": "",
          "currencySymbol": "",
          "cusBatch": "",
          "destCityCode": "769",
          "destCountry": "",
          "destDeptCode": "769K",
          "destDeptCodeMapping": "769WA",
          "destGisDeptCode": "769K",
          "destPortCode": "",
          "destPostCode": "",
          "destRouteLabel": "769K",
          "destTeamCode": "",
          "destTeamCodeMapping": "",
          "destTransferCode": "769",
          "errMsg": "",
          "expressTypeCode": "B1",
          "fbaIcon": "",
          "fileIcon": "",
          "goodsNumber": "",
          "goodsValueTotal": "",
          "icsmIcon": "",
          "limitTypeCode": "T4",
          "printFlag": "000000000",
          "printIcon": "00100000",
          "proCode": "T4",
          "proIcon": "",
          "proName": "顺丰标快",
          "sourceCityCode": "020",
          "sourceDeptCode": "",
          "sourceTeamCode": "",
          "sourceTransferCode": "",
          "twoDimensionCode": "MMM={'k1':'769','k2':'769K','k3':'','k4':'T4','k5':'SF7444600201958','k6':'','k7':'ae1f2320'}",
          "waybillNo": "SF7444600201958",
          "xbFlag": "0"
        }
      }
    ],
    "waybillNoInfoList": [
      {
        "waybillNo": "SF7444400034485",
        "waybillType": 1
      },
      {
        "waybillNo": "SF7444500019626",
        "waybillType": 2
      },
      {
        "waybillNo": "SF7444500019634",
        "waybillType": 2
      },
      {
        "waybillNo": "SF7444600201958",
        "waybillType": 3
      }
    ]
  },
  "success": true
}
```

- 失败报文-范例1

```json
{
  "success": false,
  "errorCode": 4001,
  "errorMsg": "系统发生数据错误或运行时异常",
  "msgData": null
}
}
```

###### <a id="errorCode">3.1. 错误代码 

##### 3.1 （API）平台结果代码列表

| 标识    | 说明                                                                                                    | 解决方法                                                                                                                                                                                                                                                           |
|-------|-------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| A1000 | 统一接入平台校验成功，调用后端服务成功;<br>注意：<br>不代表后端业务处理成功，实际业务处理结果，<br>需要查看响应属性apiResultData中的详细结果 | 表示接口调用正常                                                                                                                                                                                                                                                       |
| A1001 | 必传参数不可为空                                                                                              | 请做以下几点检查：<br>1、参数列表必传字段未填写<br>2、请求报文头，未配置Content-type：application/x-www-form-urlencoded <br>3、参数key存在空格问题<br>4、http请求参数都通过http URL编码传送<br>5、业务数据报文（msgData）为json报文数据格式<br>6、接口整体报文为form表单                                  |
| A1011 | OAuth2认证失败                                                                                            | 使用OAuth2认证会产生该提示<br>请检查业务接口的accessToken参数是否超过2小时，2小时口令会更新，请调用OAuth2认证接口重新获取                                                                                                                                                                              |
| A1003 | IP无效                                                                                                  | 顾客编码（partnerID）配置了需校验IP，请解除校验或按绑定IP调用接口                                                                                                                                                                                                                        |
| A1004 | 无对应服务权限                                                                                               | 可能存在的原因：<br>1、顾客编码（partnerID）没有配置（关联）对应接口的业务接口，请在【开发者对接】-【API列表】中关联；<br>2、接口请求数据与实际环境不一致，请先查看【开发者对接】-【API列表】接口状态：<br>a、【测试中】请使用沙箱环境<br>b、【已上线】请使用正式环境；<br>3、后台配置没有生效，可等待2分钟后在试试，如果还是无法操作，请报障人工处理                                 |
| A1005 | 流量受控                                                                                                  | 丰桥为接口功能联调环境，接口整体都有限流管控，单客户编码对应的每个接口限流规则：<br>1、单接口调用30次/s 2、单接口调用3000次/天<br>请尽量只操作功能联调，切勿进行接口压测，谢谢！                                                                                                                                                 |
| A1006 | 数字签名无效                                                                                                | 请做以下几点检查：<br>1、确认checkword是否配置正确<br>2、确认verifyCode、msgDigest是否加密加签正确<br>3、确认参数是否有特殊字符，如：&amp;<br>4、参数整体式form表单格式<br>5、非java类语言，需注意特殊字符，目前支持的特殊字符“*”，“空格”，“-”请优先使用<br>6、如果数字签名操作不便，可改为OAuth2认证，Token交互即可，具体参见【开发规范】【鉴权方式说明】 |
| A1007 | 重复请求                                                                                                  | 接口层暂未启用，业务层主要是针对下单接口，客户请求参数msgData中的orderId请不要重复使用，修改后在调用接口                                                                                                                                                                                                    |
| A1008 | 数据解密失败                                                                                                | 在特殊场景中使用，如有出现请报障人工处理                                                                                                                                                                                                                                           |
| A1009 | 目标服务异常或不可达                                                                                            | 接口下游服务异常，如有出现请报障人工处理                                                                                                                                                                                                                                           |
| A1010 | 状态为沙箱测试                                                                                               | 该问题在老客户中会有此类问题，新客户不会出现，如有出现请报障人工处理                                                                                                                                                                                                                             |
| A1099 | 系统异常                                                                                                  | 接口服务异常，如有出现请报障人工处理                                                                                                                                                                                                                                             |

##### 3.2 业务异常代码

| # | errorCode | 描述       | 【处理建议】           |
|---|-----------|----------|------------------|
| 1 | 8018      | 未获取到订单信息 | 确认订单号orderId是否传错 |
| 2 | 6150      | 找不到该订单   | 确认订单号orderId是否传错 |
| 3 | 6135      | 未传入订单信息  | 传入格式错误           |

[速运类接口业务相关错误码](https://open.sf-express.com/developSupport/976720?activeIndex=146623)