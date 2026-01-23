#### 订单确认/取消接口-速运类API

###### EXP_RECE_UPDATE_ORDER

------

###### 1. 功能描述

接口用于以下场景:

(1)客户在确定将货物交付给顺丰托运后，将运单上的一些重要信息，如快件重量通过此接口发送给顺丰。
(2)客户在发货前取消订单。

注意：**订单取消之后，订单号也是不能重复利用的**。

##### 2. 接口定义

###### 2.1. 公共参数

| 名称     | 值                                             |
|--------|-----------------------------------------------|
| 接口服务代码 | EXP_RECE_UPDATE_ORDER                         |
| 生产环境地址 | https://bspgw.sf-express.com/std/service      |
| 香港生产环境 | https://sfapi-hk.sf-express.com/std/service   |
| 沙箱环境地址 | https://sfapi-sbox.sf-express.com/std/service |
| 批量交易   | 不支持                                           |
| 接口类型   | 接入                                            |
| 报文类型   | JSON                                          |

**关注点：**[详见开发规范](https://open.sf-express.com/developSupport/976720)
①通讯双方采用HTTP作为通讯协议；
②提交方式为POST方式，请求头须添加"Content-type","application/x-www-form-urlencoded” 字符集编码统一使用UTF-8；
③参数需要通过http URL编码传送
④业务数据统一以字符串格式放在msgData字段中传送

###### 2.2. 公共请求参数

| 序号 | 参数列表        | 类型          | 是否必传 | 含义                                                                                                   |
|:--:|:------------|:------------|:----:|:-----------------------------------------------------------------------------------------------------|
| 1  | partnerID   | String(64)  |  是   | 合作伙伴编码/顾客编码（[获取指引](https://open.sf-express.com/developSupport/195960)）                               |
| 2  | requestID   | String(40)  |  是   | 请求唯一号UUID                                                                                            |
| 3  | serviceCode | String(50)  |  是   | 接口服务代码（EXP_RECE_UPDATE_ORDER）                                                                        |
| 4  | timestamp   | long        |  是   | 调用接口时间戳                                                                                              |
| 5  | msgDigest   | String(128) |  条件  | 数字签名,使用数字签名方式认证时必填 签名方法参考：[数字签名认证说明](https://open.sf-express.com/developSupport/976720?authId=1)     |
| 6  | accessToken | String      |  条件  | 访问令牌，使用OAuth2方式认证时必填 获取方法参考：[OAuth2认证说明](https://open.sf-express.com/developSupport/976720?authId=0) |
| 7  | msgData     | String      |  是   | 业务数据报文                                                                                               |

###### 2.3. 请求参数<msgData>

| #  | 属性名                     | 类型(约束)              | 必填 | 默认值 | 描述                                                                                                                |
|----|-------------------------|---------------------|----|-----|-------------------------------------------------------------------------------------------------------------------|
| 1  | orderId                 | String(64)          | 是  |     | 客户订单号                                                                                                             |
| 2  | dealType                | Number(1)           | 否  | 1   | 客户订单操作标识: 1:确认 2:取消 <br>丰桥下订单接口默认自动确认，不需客户重复确认；<br>如需要手工确认需要前往**控制台->查看API->下单接口->修改为不自动确认** |
| 3  | waybillNoInfoList       | List                | 条件 |     | 顺丰运单号(如dealtype=1， 必填)                                                                                            |
| 4  | customsBatchs           | String(20)          | 否  |     | 报关批次                                                                                                              |
| 5  | collectEmpCode          | String(30)          | 否  |     | 揽收员工号                                                                                                             |
| 6  | inProcessWaybillNo      | String(100)         | 否  |     | 头程运单号                                                                                                             |
| 7  | sourceZoneCode          | String(10)          | 否  |     | 原寄地网点代码                                                                                                           |
| 8  | destZoneCode            | String(10)          | 否  |     | 目的地网点代码                                                                                                           |
| 9  | totalWeight             | Number(17,5)        | 否  |     | 订单货物总重量，包含子母 件，单位千克，精确到小数点 后3位，如果提供此值，必 须>0                                                                    |
| 10 | totalVolume             | Number(16,5)        | 否  |     | 订单货物总体积，单位立方厘 米，精确到小数点后3位，会 用于计抛（是否计抛具体商务 沟通中双方约定）                                                                |
| 11 | expressTypeId           | Number(5)           | 否  |     | 快件产品类别，支持附录《快 件产品类别表》的产品编码 值，仅可使用与顺丰销售约定 的快件产品                                                                    |
| 12 | extraInfoList           | List                | 否  |     | 扩展属性                                                                                                              |
| 13 | totalLength             | Number(16, 5)       | 否  |     | 客户订单货物总长，单位厘米， 精确到小数点后3位，包含子 母件                                                                                   |
| 14 | totalWidth              | Number(16, 5)       | 否  |     | 客户订单货物总宽，单位厘米， 精确到小数点后3位，包含子 母件                                                                                   |
| 15 | totalHeight             | Number(16, 5)       | 否  |     | 客户订单货物总高，单位厘米， 精确到小数点后3位，包含子 母件                                                                                   |
| 16 | serviceList             | List                | 否  |     | 增值服务信息                                                                                                            |
| 17 | isConfirmNew            | Number (1)          | 否  |     | 是否走新通用确认1：支持修改联系人 2：支持改其他客户订单默认0                                                                                  |
| 18 | destContactInfo         | OrderContactInfoDto | 否  |     | 收件人信息                                                                                                             |
| 19 | isDocall                | Number(1)           | 否  |     | 是否通过手持终端通知顺丰收派员上门收件， 支持以下值：1：要求其它为不要求                                                                             |
| 20 | specialDeliveryTypeCode | String(3)           | 否  |     | 1. 特殊派送类型代码 身份验证 2. 极效前置单                                                                                         |
| 21 | specialDeliveryValue    | String(100)         | 否  |     | 1> 特殊派件具体表述 证件类型:证件后8位 如：1:09296231（1表示身份证，暂不支持其他证件） 2>.极效前置单时:**Y**:若不支持则返回普通运单**N**:若不支持则返回错误码            |
| 22 | sendStartTm             | Date                | 否  |     | 预约时间(上门揽收时间)                                                                                                      |
| 23 | pickupAppointEndtime    | Date                | 否  |     | 上门揽收截止时间                                                                                                          |
| 24 | remark                  | 	String(100)        | 	否 | 	   | 	备注                                                                                                               |

###### 2.3.1 元素<请求> OrderUpdate/waybillNoInfoList

| # | 属性名         | 类型(约束)        | 必填 | 默认值 | 描述                       |
|---|-------------|---------------|----|-----|--------------------------|
| 1 | waybillType | Number(1)     | 否  |     | 运单号类型 1：母单 2 :子单 3 : 签回单 |
| 2 | waybillNo   | String(15)    | 否  |     | 运单号                      |
| 3 | boxNo       | String(64)    | 否  |     | 箱号                       |
| 4 | length      | Number (16,3) | 否  |     | 长                        |
| 5 | width       | Number (16,3) | 否  |     | 宽                        |
| 6 | height      | Number (16,2) | 否  |     | 高                        |
| 7 | weight      | Number (16,2) | 否  |     | 重量(kg)                   |

**说明：**
1、当包裹列表List<WaybillNoInfo>信息里长宽高重量任一有值的时候，取包裹信息里计重之和{SUM（max（长宽高(length* width*
height/轻抛系数)，重量weight}与总重量totalWeight进行计重（取最大值）

2、当包裹列表List<WaybillNoInfo>信息里长宽高重量都无值，取运单总长宽高(totalLength* totalWidth*
totalHeight/轻抛系数)、总重量totalWeight用于计重（取最大值）；（另外：当总长宽高不全时，取运单总体积(totalVolume/轻抛系数)
、总重量totalWeight用于计重（取最大值））

###### 2.3.2 元素<请求> OrderUpdate/extraInfoList

| # | 属性名      | 类型(约束)       | 必填 | 默认值 | 描述                                            |
|---|----------|--------------|----|-----|-----------------------------------------------|
| 1 | attrName | String(256)  | 否  |     | 扩展字段 说明： attrName为字段定义， 具体如下表，value存在 attrVal |
| 2 | attrVal  | String(1024) | 否  |     | 扩展字段值                                         |

###### 扩展字段备注

| attrName          | attrVal                                                                                                                                        |
|-------------------|------------------------------------------------------------------------------------------------------------------------------------------------|
| attr001           |                                                                                                                                                |
| attr002           |                                                                                                                                                |
| userId            | 商家或合作店铺id 丰网业务                                                                                                                                 |
| branchCode        | 丰网合作网点（丰网必填) 丰网业务                                                                                                                              |
| branchAddressId   | 丰网合作地址id 丰网业务                                                                                                                                  |
| channelCode       | 渠道编码 丰网业务                                                                                                                                      |
| subsidySource     | 国补标识，dy为抖音；sn为苏宁；tb为淘天；pdd为拼多多；ks为快手；wph为唯品会；dw为得物；xhs为小红书；sph为视频号；merchant为商家自营国补; cancel为取消标识；                                               |
| infoCollectSource | 采集服务标识，dy为抖音；sn为苏宁；tb为淘天；pdd为拼多多；ks为快手；wph为唯品会；dw为得物；xhs为小红书；sph为视频号；merchant为商家自营采集服务; cancel为取消标识                                            |
| wpMerchantCode    | 微派商户编码，微派会给每个接入客户提供一个唯一编码<br> **测试可用：** B2019020002-0014                                                                                 |
| wpServiceCode     | 微派任务编码，生产环境需与微派申请任务编码<br>**测试可用：**<br>Pro24125839--手机<br>Pro24125847--平板<br>Pro24125848--电脑<br>Pro24125849--智能穿戴 |
| wpExtJson         | "{"sn":"123456","imei":"123456"}" ---sn为sn编码，只支持一个，imei为手机/平板emei号，只支持一个                                                                       |

**国补传参示例（信息采集由subsidySource调整为infoCollectSource，其他不变）**

```json
"extraInfoList":[
{
"attrName": "subsidySource",
"attrVal": "sn"---国补标识，dy为抖音；sn为苏宁；tb为淘天；pdd为拼多多；ks为快手；wph为唯品会；dw为得物；xhs为小红书；sph为视频号；merchant为商家自营国补; cancel为取消标识；
},
{
"attrName": "wpMerchantCode",
"attrVal": "B2019020002-0014" ---微派商户编码，微派会给每个接入客户提供一个唯一编码
},
{
"attrName": "wpServiceCode",
"attrVal": "Pro24123096" ---微派任务编码，类型为手机，传ProXXXXX，类型为平板，传ProXXXXX，需与微派申请任务编码
},
{
"attrName": "wpExtJson",
"attrVal": "{\"imei\": \"8621111111192\",\"sn\": \"2MH11111112368\"}" ---sn为sn编码，只支持一个，imei为手机/平板emei号，只支持一个
}
]

```

**说明：**</br>①国补传值要求sn/imei是否传入参考业务场景定义，原则上3C品类国补必传。</br>②沙箱环境直传入字段即可触发，可以使用示例报文请求验证

###### 2.3.3. 元素<请求> OrderUpdate/serviceList

| # | 属性名    | 类型(约束)     | 必填 | 默认值 | 描述                   |
|---|--------|------------|----|-----|----------------------|
| 1 | name   | String(20) | 是  |     | 增值服务名，如COD等          |
| 2 | value  | String(30) | 条件 |     | 增值服务扩展属性，参考增值 服务传值说明 |
| 3 | value1 | String(30) | 条件 |     | 增值服务扩展属性1            |
| 4 | value2 | String(30) | 条件 |     | 增值服务扩展属性2            |
| 5 | value3 | String(30) | 条件 |     | 增值服务扩展属性3            |
| 6 | value4 | String(30) | 条件 |     | 增值服务扩展属性4            |

2.3.4 元素<请求> OrderUpdate/List

| # | 属性名    | 类型（约束）     | 必填 | 默认值 | 描述                   |
|---|--------|------------|----|-----|----------------------|
| 1 | name   | String(20) | 是  |     | 增值服务名，如COD等。         |
| 2 | value  | String(30) | 条件 |     | 增值服务扩展属性，参考增值服务传值说明。 |
| 3 | value1 | String(30) | 条件 |     | 增值服务扩展属性             |
| 4 | value2 | String(30) | 条件 |     | 增值服务扩展属性2            |
| 5 | value3 | String(30) | 条件 |     | 增值服务扩展属性3            |
| 6 | Value4 | String(30) | 条件 |     | 增值服务扩展属性4            |

2.3.5 元素<请求> OrderUpdate/OrderContactInfoDto

| # | 属性名      | 类型（约束）      | 必填 | 默认值 | 描述                                      |
|---|----------|-------------|----|-----|-----------------------------------------|
| 1 | company  | String(100) | 条件 |     | 公司名称                                    |
| 2 | contact  | String(100) | 条件 |     | 联系人                                     |
| 3 | tel      | String(20)  | 条件 |     | 收方电话                                    |
| 4 | mobile   | String(20)  | 否  |     | 收方手机                                    |
| 5 | country  | String(30)  | 是  |     | 国家或地区 2位代码参照附录国家代码附件                    |
| 6 | province | String(30)  | 否  |     | 所在省级行政区名称如：北京、广东省、广西壮族自治区等              |
| 7 | city     | String(100) | 否  |     | 所在地级行政区名称，必须是标准的城市称谓 如：北京市、深圳市、大理白族自治州等 |
| 8 | county   | String(30)  | 否  |     | 所在县/区级行政区名称，必须是标准的县/区称谓，如：福田区           |
| 9 | address  | String(200) | 条件 |     | 详细地址，若province/city字段的值不传，此字段必须包含省市     |

###### 2.4. 公共响应参数

| # | 属性名       | 类型(约束) | 必填 | 默认值 | 描述                   |
|---|-----------|--------|----|-----|----------------------|
| 1 | success   | String | 是  |     | true 请求成功，false 请求失败 |
| 2 | errorCode | String | 是  |     | 错误编码，S0000成功         |
| 3 | errorMsg  | String | 是  |     | 错误描述                 |
| 4 | msgData   | String | 是  |     | 返回的详细数据              |

###### 2.5. 响应参数<msgData>

| # | 属性名               | 类型(约束)     | 必填 | 默认值 | 描述                         |
|---|-------------------|------------|----|-----|----------------------------|
| 1 | orderId           | String(64) | 是  |     | 客户订单号                      |
| 2 | waybillNoInfoList | List       | 否  |     | 顺丰运单号                      |
| 3 | resStatus         | Number(1)  | 是  |     | 备注 1:客户订单号与顺丰运单不匹配 2 :操作成功 |
| 4 | extraInfoList     | List       | 否  |     | 扩展属性                       |

###### 2.6. 请求示例\应用场景(JSON)示例

请求报文（订单确认）:

```
{
    "dealType": 1,
    "orderId": "BZL51054473992769999",
    "totalHeight": 29.98,
    "totalLength": 29.98,
    "totalVolume": 26946.035992000005,
    "totalWeight": 2.09,
    "totalWidth": 29.98,
    "waybillNoInfoList": [
        {
            "waybillNo": "SF2000090670189",
            "waybillType": 1
        }
    ]
}
```

请求报文（订单取消）:

```
{
    "dealType": 2,
    "language": "zh-CN",
    "orderId": "eb21c793-a45a-4d1e-9a2e-1b6e0cd49668",
    "totalWeight": 1,
    "waybillNoInfoList": []
}
```

###### 2.7. 返回示例\应用场景(JSON)示例

响应报文:

- 成功响应(订单确认成功):

```
{
    "apiResponseID": "000173271983983FEDCAD1803FA64A3F",
    "apiErrorMsg": "",
    "apiResultCode": "A1000",
    "apiResultData": "{\"success\":true,\"errorCode\":\"S0000\",\"errorMsg\":null,\"msgData\":{\"orderId\":\"5ed9696e-a81d-4a5b-968d-182c2d8c09e0\",\"waybillNoInfoList\":[{\"waybillType\":1,\"waybillNo\":\"SF7444400048449\"}],\"resStatus\":2,\"extraInfoList\":null}}"
}
```

- 失败响应(订单确认失败)：

```
{
    "apiResponseID": "000173271968963FA47C03E68000103F",
    "apiErrorMsg": "",
    "apiResultCode": "A1000",
    "apiResultData": "{\"success\":false,\"errorCode\":\"8024\",\"errorMsg\":\"未下单\",\"msgData\":null}"
}
```

响应报文:

- 成功响应(订单取消成功):

```
{
    "success": true,
    "errorCode": "S0000",
    "errorMsg": null,
    "msgData": {
        "orderId": "eb21c793-a45a-4d1e-9a2e-1b6e0cd49668",
        "waybillNoInfoList": [{
            "waybillType": 1,
            "waybillNo": "SF7444400043064"
        }],
        "resStatus": 2,
        "extraInfoList": null
    }
}
```

- 失败响应(订单取消失败)：

```
{
    "apiErrorMsg": "",
    "apiResponseID": "00019AC8457F7D3FC372D7B021E1603F",
    "apiResultCode": "A1000",
    "apiResultData": "{\"success\":false,\"errorCode\":\"6118\",\"errorMsg\":\"订单号不能为空\",\"msgData\":null}"
}
```

###### 3.1. 错误代码

##### 3.1 （API）平台结果代码列表

| 标识    | 说明                                                                        | 【处理建议】                                                                                                      |
|-------|---------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------|
| A1000 | 统一接入平台校验成功，调用后端服务成功； 注意：不代表后端业务处理成功，实际业务处理结果， 需要查看响应属性apiResultData中的详细结果 |                                                                                                             |
| A1001 | 必传参数不可为空                                                                  | serviceCode partnerID requestID timestamp msgDigest msgData 不可为空                                            |
| A1002 | 请求时效已过期                                                                   | 时效参考auth2 https://open.sf-express.com/customerService/395002?interId=590549&amp;faqId=4                     |
| A1003 | IP无效                                                                      | 参考常见问题 https://open.sf-express.com/customerService/395002?activeIndex=905584&amp;interId=590549&amp;faqId=2 |
| A1004 | 无对应服务权限                                                                   | 联系销售经理，配置权限                                                                                                 |
| A1005 | 流量受控                                                                      | 测试环境流量限制为5000，请不要在测试环境做压测                                                                                   |
| A1006 | 数字签名无效                                                                    | 参考常见问题 签名加解密问题 https://open.sf-express.com/customerService/395002?activeIndex=905584&amp;interId=795986     |
| A1007 | 重复请求                                                                      | 过一分钟在尝试                                                                                                     |
| A1008 | 数据解密失败                                                                    |                                                                                                             |
| A1009 | 目标服务异常或不可达                                                                |                                                                                                             |
| A1099 | 系统异常                                                                      |                                                                                                             |

##### 3.2 业务异常代码

| #  | errorCode | 描述               | 【处理建议】                                 |
|----|-----------|------------------|----------------------------------------|
| 1  | 20052     | 月结卡号不匹配 不允许操作该订单 | 月结卡号跟传入的不匹配，修改月结卡号匹配后，才能确认             |
| 2  | 8019      | 订单已确认或已消单        | 确认下单模板是否配置为下单自动确认                      |
| 3  | 8018      | 未获取到订单信息         | 修改订单号orderId                           |
| 4  | 6136      | 未传入订单确认信息        | 传入报文格式有问题                              |
| 5  | 8037      | 已消单              | 订单已经取消，再次取消会报错                         |
| 6  | 8017      | 订单号与运单号不匹配       | 检查传入的订单号orderId跟运单号是否匹配                |
| 7  | 8228      | 传入增值服务不能通过确认接口修改 | 对应增值服务不支持确认接口修改                        |
| 8  | 8253      | 订单已取消            | 订单已经取消，再次取消会报错                         |
| 9  | 8252      | 订单已确认            | 订单已经确认，再次确认会报错                         |
| 10 | 20034     | 预约时间必须大于当前时间     | sendStartTm要大于当前时间                     |
| 11 | 8267      | 新预约时间必须在70分钟后    | 确认接口传入的预约时间sendStartTm必须大于 （当前时间+70分钟） |

[速运类接口业务相关错误码](https://open.sf-express.com/developSupport/976720?activeIndex=146623) 