#### 下订单接口-速运类API（国内件、大陆港澳台互寄件）

###### EXP_RECE_CREATE_ORDER

------------

###### 1. 功能描述

- 下订单接口根据客户需要,可提供以下五个功能:

  ●
  客户系统向顺丰下发订单（大陆寄往大陆、港澳台，港澳台寄往大陆或港澳台），其他国际流向请对接[国际API](https://open.sf-express.com/Api?category=101&amp;apiClassify=1)

  ● 为订单分配运单号。

  ● 筛单【**关注响应字段filterResult筛单结果**（筛单结果： 1：人工确认 2：可收派 3：不可以收派 4：无法确定 ）】。

  ● 云打印面单推送（可选）。
  提供给用户自主打单的服务，打印好的面单会直接推送到用户配置的地址

  ● 路由轨迹，对接路由推送接口后下单，有路由轨迹数据产生会自动推送到路由推送接口配置的URL地址

注：专线对接客户请提前联系客户经理确认对接方案后再进行对接。

##### 2. 接口定义

###### 2.1. 公共参数

| 名称     | 值                                             |
|--------|-----------------------------------------------|
| 接口服务代码 | EXP_RECE_CREATE_ORDER                         |
| 生产环境地址 | https://bspgw.sf-express.com/std/service      |
| 香港生产环境 | https://sfapi-hk.sf-express.com/std/service   |
| 沙箱环境地址 | https://sfapi-sbox.sf-express.com/std/service |
| 批量交易   | 不支持                                           |
| 接口类型   | 接入                                            |
| 报文类型   | JSON（msgData部分为JSON，整体为表单格式传输）                |

**关注点：**[详见开发规范](https://open.sf-express.com/developSupport/976720)
①通讯双方采用HTTP作为通讯协议；
②提交方式为POST方式，请求头须添加"Content-type","application/x-www-form-urlencoded” 字符集编码统一使用UTF-8；
③参数需要通过http URL编码传送
④业务数据统一以字符串格式放在msgData字段中传送

###### <a id="commonReqParam">2.2. 公共请求参数

| 序号 | 参数列表        | 类型          | 是否必传 | 含义                                                                                                                                  |
|:--:|:------------|:------------|:----:|:------------------------------------------------------------------------------------------------------------------------------------|
| 1  | partnerID   | String(64)  |  是   | 合作伙伴编码/顾客编码（[获取指引](https://open.sf-express.com/developSupport/195960)）                                                              |
| 2  | requestID   | String(40)  |  是   | 请求唯一号UUID                                                                                                                           |
| 3  | serviceCode | String(50)  |  是   | 接口服务代码                                                                                                                              |
| 4  | timestamp   | long        |  是   | 调用接口时间戳                                                                                                                             |
| 5  | msgDigest   | String(128) |  条件  | 数字签名,使用数字签名方式认证时必填，不可与accessToken字段同时传参    <br/>签名方法参考：[数字签名认证说明](https://open.sf-express.com/developSupport/976720?authId=1) |
| 6  | accessToken | String      |  条件  | 访问令牌，使用OAuth2方式认证时必填，不可与msgDigest同时传参 <br/>获取方法参考：[OAuth2认证说明](https://open.sf-express.com/developSupport/976720?authId=0)    |
| 7  | msgData     | String      |  是   | 业务数据报文                                                                                                                              |

###### 2.3. 请求参数<msgData> 

###### 2.3.1  元素<请求> Order

| 序号 | 属性名                         | 类型(约束)       | 必填 | 默认值         | 描述                                                                                                                                                        |
|----|-----------------------------|--------------|----|-------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1  | language                    | String(10)   | 是  |             | 响应报文的语言， 缺省值为zh-CN，目前支持以下值zh-CN 表示中文简体， zh-TW或zh-HK或 zh-MO表示中文繁体， en表示英文                                                                                  |
| 2  | orderId                     | String(64)   | 是  |             | 客户订单号，**重复使用订单号时返回第一次下单成功时的运单信息**                                                                                                                         |
| 3  | waybillNoInfoList           | List         | 否  |             | 顺丰运单号                                                                                                                                                     |
| 4  | customsInfo                 | CustomsInfo  | 否  |             | 报关信息，查看[《海关配置流程指引》](https://open.sf-express.com/developSupport/195960?activeIndex=644140)                                                                 
| 5  | cargoDetails                | List         | 是  |             | 托寄物信息                                                                                                                                                     |
| 6  | cargoDesc                   | String(20)   | 否  |             | 拖寄物类型描述,如： 文件，电子产品，衣服等，</br>**托寄物名称请使用cargoDetails传值**                                                                                              |
| 7  | extraInfoList               | List         | 否  |             | 扩展属性                                                                                                                                                      |
| 8  | serviceList                 | List         | 否  |             | 增值服务信息，支持附录： [《增值服务产品表》](https://open.sf-express.com/developSupport/734349?activeIndex=313317)                                                            |
| 9  | contactInfoList             | List         | 是  |             | 收寄双方信息                                                                                                                                                    |
| 10 | monthlyCard                 | String(20)   | 条件 |             | 顺丰月结卡号 月结支付时传值，现结不需传值；沙箱联调可使用测试月结卡号7551234567（非正式，无须绑定，仅支持联调使用）<br>**生产月结请前往：控制台-开发者/ERP对接->应用详情->绑定月结**                                      |
| 11 | payMethod                   | Number(2)    | 否  | 1           | 付款方式，支持以下值： 1:寄方付 2:收方付 3:第三方付                                                                                                                            |
| 12 | expressTypeId               | Number(5)    | 是  | 1           | 快件产品类别，支持附录《[快件产品类别表](https://open.sf-express.com/developSupport/734349?activeIndex=324604)》的产品编码值，仅可使用与顺丰销售约定的快件产品。                                      |
| 13 | parcelQty                   | Number(5)    | 否  | 1           | 包裹数，一个包裹对应一个运单号；若包裹数大于1，则返回一个母运单号和N-1个子运单号                                                                                                                |
| 14 | totalLength                 | Number(16,5) | 否  |             | 客户订单货物总长，单位厘米， 精确到小数点后3位， 包含子母件                                                                                                                           |
| 15 | totalWidth                  | Number(16,5) | 否  |             | 客户订单货物总宽，单位厘米， 精确到小数点后3位， 包含子母件                                                                                                                           |
| 16 | totalHeight                 | Number(16,5) | 否  |             | 客户订单货物总高，单位厘米， 精确到小数点后3位， 包含子母件                                                                                                                           |
| 17 | totalVolume                 | Number(16,5) | 否  |             | 订单货物总体积，单位立方厘米, 精确到小数点后3位，会用于计抛 (是否计抛具体商务沟通中 双方约定)                                                                                                        |
| 18 | totalWeight                 | Number(17,5) | 条件 |             | 订单货物总重量（郑州空港海关必填）， 若为子母件必填， 单位千克， 精确到小数点后3位，如果提供此值， 必须>0 (子母件需>6)                                                                                   |
| 19 | totalNetWeight              | Number(17,5) | 否  |             | 商品总净重                                                                                                                                                     |
| 20 | sendStartTm                 | Date         | 否  | 接收 到报 文的 时间 | 要求上门取件开始时间， 格式： YYYY-MM-DD HH24:MM:SS， 示例： 2012-7-30 09:30:00 ，若该字段没有赋值，默认开始时间为当前时间，（可配合上门取件截止时间pickupAppointEndTime**扩展字段备注**进行下发，若没有给截止时间则系统默认1小时的截止时间） |
| 21 | isDocall                    | Number(1)    | 否  | 0           | 支持以下值： 1：要求 0：不要求。<br>为1时系统分配揽收任务给到小哥，小哥需要按照预约时间上门揽收，一般用于高时效产品。<br>为0时是集收场景，发货商家提前与网点小哥约定每天揽收时段，一般用于集收场景。                                     |
| 22 | isSignBack                  | Number(1)    | 否  | 0           | 是否返回签回单 （签单返还）的运单号， 支持以下值： 1：要求 0：不要求                                                                                                                     |
| 23 | custReferenceNo             | String(100)  | 否  |             | 客户参考编码：如客户原始订单号                                                                                                                                           |
| 24 | temperatureRange            | Number(2)    | 条件 |             | 温度范围类型，当 express_type为12 医药温控件 时必填，支持以下值： 1:冷藏 3：冷冻                                                                                                       |
| 25 | orderSource                 | String(50)   | 否  |             | 订单平台类型 （对于平台类客户， 如果需要在订单中 区分订单来源， 则可使用此字段） 天猫:tmall， 拼多多：pinduoduo， 京东 : jd 等平台类型编码                                                                       |
| 27 | remark                      | String(100)  | 否  |             | 备注                                                                                                                                                        |
| 28 | isOneselfPickup             | Number(1)    | 否  | 0           | 快件自取，支持以下值： 1：客户同意快件自取 0：客户不同意快件自取                                                                                                                        |
| 29 | filterField                 | String       | 否  |             | 筛单特殊字段用来人工筛单                                                                                                                                              |
| 30 | isReturnQRCode              | Number(1)    | 否  | 0           | 是否返回用来退货业务的 二维码URL， 支持以下值： 1：返回二维码 0：不返回二维码                                                                                                               |
| 31 | specialDeliveryTypeCode     | String(3)    | 否  |             | 当EXPRESS_TYPE=235 2：极效前置单（当日达） 5：极效前置小时达 当EXPRESS_TYPE=265 4：预售电标                                                                                         
| 32 | specialDeliveryValue        | String(100)  | 否  |             | 特殊派件具体表述 证件类型: 证件后8位如： 1:09296231（1 表示身份证， 暂不支持其他证件）                                                                                                      |
| 33 | merchantPayOrderNo          | String(100)  | 否  |             | 商户支付订单号                                                                                                                                                   |
| 34 | isReturnSignBackRoute label | Number(1)    | 否  | 0           | 是否返回签回单路由标签： 默认0， 1：返回路由标签， 0：不返回                                                                                                                         |
| 35 | isReturnRoutelabel          | Number(1)    | 是  | 1           | 是否返回路由标签： 默认1， 1：返回路由标签， 0：不返回；除部分特殊用户外，其余用户都默认返回                                                                                                         |
| 36 | isUnifiedWaybillNo          | Number(1)    | 否  | 0           | 是否使用国家统一面单号 1：是（建议使用，返回SF开头的15位运单号）， 0：否(特殊诉求才用，返回12位的运单号)                                                                                                |
| 37 | podModelAddress             | String(1024) | 否  |             | 签单返还范本地址                                                                                                                                                  |
| 38 | inProcessWaybillNo          | String(100)  | 否  |             | 头程运单号（郑州空港海关必填）                                                                                                                                           |
| 39 | isGenWaybillNo              | Number(1)    | 否  | 1           | 是否需求分配运单号1：分配 0：不分配（若带单号下单，请传值0）                                                                                                                          |
| 40 | scenePlanCode               | String(50)   | 条件 |             | 组件服务编码，特定业务场景使用</br>**该字段与快件产品类别 expressTypeId 二选一，组件服务编码需提前与客户经理沟通确定**                                                                             |

###### 2.3.2 元素<请求>Order/List<**ContactInfo**>

| #  | 属性名           | 类型(约束)       | 必填 | 描述                                                                                                                                           |
|----|---------------|--------------|----|----------------------------------------------------------------------------------------------------------------------------------------------|
| 1  | contactType   | Number(1)    | 是  | 地址类型： 1，寄件方信息 2，到件方信息                                                                                                                        |
| 2  | company       | String(100)  | 条件 | 公司名称                                                                                                                                         |
| 3  | contact       | String(100)  | 条件 | 联系人                                                                                                                                          |
| 4  | tel           | String(20)   | 条件 | 联系电话（tel和mobile字段必填其中一个）                                                                                                                     |
| 5  | mobile        | String(20)   | 条件 | 手机（tel和mobile字段必填其中一个）                                                                                                                       
| 6  | country       | String(30)   | 是  | 国家或地区代码 例如：内地件CN 香港852                                                                                                                       |
| 7  | province      | String(30)   | 否  | 所在省级行政区名称，必须是标准的省级行政区名称如：北 京、广东省、广西壮族自治区等；此字段影响原寄地代码识 别，建议尽可能传该字段的值                                                                          |
| 8  | city          | String(100)  | 否  | 所在地级行政区名称，必须是标准的城市称谓 如：北京市、 深圳市、大理白族自治州等； 此字段影响原寄地代码识别， 建议尽可能传该字段的值                                                                          |
| 9  | county        | String(30)   | 否  | 所在县/区级行政区名称，必须 是标准的县/区称谓，如：福田区，南涧彝族自治县、准格尔旗等                                                                                                 |
| 10 | address       | String(200)  | 是  | 详细地址，若有四级行政区划，如镇/街道等信息可拼接至此字段，格式样例：镇/街道+详细地址。若province/city 字段的值不传，此字段必须包含省市信息，避免影响原寄地代码识别，如：广东省深圳市福田区新洲十一街万基商务大厦10楼；此字段地址必须详细，否则会影响目的地中转识别； |
| 11 | postCode      | String(25)   | 条件 | 邮编，跨境件必填（中国内地， 港澳台互寄除外）                                                                                                                      |
| 12 | email         | String(200)  | 否  | 邮箱地址                                                                                                                                         |
| 13 | taxNo         | String(100)  | 否  | 税号                                                                                                                                           |
| 14 | contactRemark | String(100)  | 否  | 联系人属性（01：个人件，02：公司件,跨境件或国际件需要)                                                                                                               |
| 15 | certType      | String(200)  | 否  | 证件类型(跨境件或国际件需要)，参考 [《证件类型说明》](https://open.sf-express.com/developSupport/734349?activeIndex=720345)                                          |
| 16 | certNo        | String(1000) | 否  | 证件号码(跨境件或国际件需要)，参考[《证件类型说明》](https://open.sf-express.com/developSupport/734349?activeIndex=720345)                                           |

###### 2.3.3 元素<请求>Order/List<**CargoDetail**>

| #  | 属性名                           | 类型(约束)        | 必填 | 描述                                                                                            |
|----|-------------------------------|---------------|----|-----------------------------------------------------------------------------------------------|
| 1  | name                          | String(128)   | 是  | 货物名称，如果需要生成电子 运单，则为必填                                                                         |
| 2  | count                         | Number(5)     | 条件 | 货物数量 跨境件报关需要填写                                                                                |
| 3  | unit                          | String(30)    | 条件 | 货物单位，如：个、台、本， 跨境件报关需要填写                                                                       |
| 4  | weight                        | Number(16,3)  | 条件 | 订单货物单位重量，包含子母件， 单位千克，精确到小数点后3位 跨境件报关需要填写                                                      |
| 5  | amount                        | Number(17,3)  | 条件 | 货物单价，精确到小数点后3位， 跨境件报关需要填写                                                                     |
| 6  | currency                      | String(5)     | 条件 | 货物单价的币别： 参照附录[《国际件币别表》](https://open.sf-express.com/developSupport/734349?activeIndex=482730) |
| 7  | sourceArea                    | String(5)     | 条件 | 原产地国别， 跨境件报关需要填写                                                                              |
| 8  | productRecordNo               | String(18)    | 否  | 货物产品国检备案编号                                                                                    |
| 9  | goodPrepardNo                 | String(100)   | 否  | 商品海关备案号                                                                                       |
| 10 | taxNo                         | String(100)   | 否  | 商品行邮税号                                                                                        |
| 11 | hsCode                        | String(100)   | 否  | 海关编码                                                                                          |
| 12 | goodsCode                     | String(60)    | 否  | 商品编号                                                                                          |
| 13 | brand                         | String(60)    | 否  | 货物品牌                                                                                          |
| 14 | specifications                | String(60)    | 否  | 货物规格型号                                                                                        |
| 15 | manufacturer                  | String(100)   | 否  | 生产厂家                                                                                          |
| 16 | shipmentWeight                | Double (16,3) | 否  | 托寄物毛重                                                                                         |
| 17 | length                        | Double (16,3) | 否  | 托寄物长                                                                                          |
| 18 | width                         | Double (16,3) | 否  | 托寄物宽                                                                                          |
| 19 | height                        | Double (16,3) | 否  | 托寄物高                                                                                          |
| 20 | volume                        | Double (16,2) | 否  | 托寄物体积                                                                                         |
| 21 | cargoDeclaredValue            | Double (16,5) | 否  | 托寄物声明价值（杭州口岸必填）                                                                               |
| 22 | declaredValueDeclaredCurrency | String(5)     | 否  | 托寄物声明价值币别（杭州口岸必填）                                                                             |
| 23 | cargoId                       | String(60)    | 否  | 货物id（逆向物流）                                                                                    |
| 24 | intelligentInspection         | Number(1)     | 否  | 智能验货标识(1-是,0-否) （逆向物流）                                                                        |
| 25 | snCode                        | String(4000)  | 否  | 货物标识码（逆向物流）                                                                                   |
| 26 | stateBarCode                  | String(50)    | 否  | 国条码                                                                                           |

###### 2.3.4  元素<请求>Order/List<**Service**>

| # | 属性名    | 类型(约束)     | 必填 | 默认值 | 描述                                                                                                  |
|---|--------|------------|----|-----|-----------------------------------------------------------------------------------------------------|
| 1 | name   | String(20) | 是  |     | 增值服务名，如COD等，支持附录： [《增值服务产品表》](https://open.sf-express.com/developSupport/734349?activeIndex=313317) |
| 2 | value  | String(30) | 条件 |     | 增值服务扩展属性，参考增值 服务传值说明                                                                                |
| 3 | value1 | String(30) | 条件 |     | 增值服务扩展属性                                                                                            |
| 4 | value2 | String(30) | 条件 |     | 增值服务扩展属性2                                                                                           |
| 5 | value3 | String(30) | 条件 |     | 增值服务扩展属性3                                                                                           |
| 6 | value4 | String(30) | 条件 |     | 增值服务扩展属性4                                                                                           |

###### 2.3.5  元素<请求>Order/CustomsInfo

| # | 属性名                   | 类型(约束)        | 必填 | 默认值                  | 描述                                       |
|---|-----------------------|---------------|----|----------------------|------------------------------------------|
| 1 | declaredValue         | Number(16, 5) | 条件 |                      | 客户订单货物总声明价值， 包含子母件，精确到小数点 后3位。如果是跨境件，则必填 |
| 2 | declaredValueCurrency | String(5)     | 否  | 中国内地 默认CNY， 否则 默认USD | 货物声明价值币别，跨境 件报关需要填写 参照附录币别代码附件           |
| 3 | customsBatchs         | String(20)    | 否  |                      | 报关批次                                     |
| 4 | taxPayMethod          | Number(2)     | 否  |                      | 税金付款方式，支持以下值： 1:寄付 2：到付 3. 第三方付          |
| 5 | taxSettleAccounts     | String(30)    | 否  |                      | 税金结算账号                                   |
| 6 | paymentTool           | String(100)   | 否  |                      | 支付工具                                     |
| 7 | paymentNumber         | String(100)   | 否  |                      | 支付号码                                     |
| 8 | orderName             | String(100)   | 否  |                      | 客户订单下单人姓名                                |
| 9 | tax                   | String(10)    | 否  |                      | 税款                                       |

###### 2.3.6  元素<请求>Order/List<**WaybillNoInfo**>

| # | 属性名         | 类型(约束)        | 必填 | 默认值 | 描述                                             |
|---|-------------|---------------|----|-----|------------------------------------------------|
| 1 | waybillType | Number (1)    | 条件 |     | 运单号类型1：母单 2 :子单 3 : 签回单*waybillNoInfoList有值时必传 
| 2 | waybillNo   | String(15)    | 否  |     | 运单号                                            
| 3 | boxNo       | String(64)    | 否  |     | 箱号                                             
| 4 | length      | Number (16,3) | 否  |     | 长(cm)                                          
| 5 | width       | Number (16,3) | 否  |     | 宽(cm)                                          
| 6 | height      | Number (16,2) | 否  |     | 高(cm)                                          
| 7 | weight      | Number (16,2) | 否  |     | 重量(kg)                                         

**说明：**
1、当包裹列表List<WaybillNoInfo>信息里长宽高重量任一有值的时候，取包裹信息里计重之和{SUM（max（长宽高(length* width*
height/轻抛系数)，重量weight}与总重量totalWeight进行计重（取最大值）

2、当包裹列表List<WaybillNoInfo>信息里长宽高重量都无值，取运单总长宽高(totalLength* totalWidth*
totalHeight/轻抛系数)、总重量totalWeight用于计重（取最大值）；（另外：当总长宽高不全时，取运单总体积(totalVolume/轻抛系数)
、总重量totalWeight用于计重（取最大值））

###### 2.3.7 元素<请求>Order/List<**ExtraInfo**>

| # | 属性名      | 类型(约束)       | 必填 | 默认值 | 描述                                         |
|---|----------|--------------|----|-----|--------------------------------------------|
| 1 | attrName | String(256)  | 否  |     | 扩展字段说明：attrName为字段定义， 具体如下表，value存在attrVal |
| 2 | attrVal  | String(1024) | 否  |     | 扩展字段值                                      |

###### 2.3.7.1 扩展字段备注

| attrName             | attrVal                                                                                                                                                                        |
|----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| attr001              | 物品件数（杭州口岸必填）                                                                                                                                                                   |
| attr002              | 物品净重，郑州空港口岸, 郑州综保税必须字段，最多保留小数点后四位，单位Kg                                                                                                                                         |
| attr003              | 进出口时间（郑州空港口岸必须字段格式：yyyy-MM-dd hh:mm:ss）                                                                                                                                        |
| attr005              | 商品运输费用,无则填0（郑州综保,杭州海关,重庆口岸 要求字段）                                                                                                                                               |
| attr006              | 报关批次（杭州口岸必填，yyyy-dd-mm）                                                                                                                                                        |
| attr007              | 商品保险费用，无则填0（郑州综保,杭州海关,重庆口岸 要求字段）                                                                                                                                               |
| attr009              | 杭州海关版本代码：03                                                                                                                                                                    |
| attr010              | 电商企业十位编码，郑州综保字段                                                                                                                                                                |
| attr014              | 签回单标识，对应值为“singBackInfo”                                                                                                                                                       |
| attr015              | 签回单标识，结合attr014使用，英文逗号分隔，对应值attrVal：1:签名，2:盖章，3:登记身份证号，4:收取身份证复印件，5、【收取派件存根】(香港专用） 【其他文件】(香港专用）7、【签收日期】8、【电话号码】，英文逗号分隔                                                         |
| pickupAppointEndTime | 上门揽收截止时间，格式: 2023-07-04 12:01:02（预约单传预约截止时间，不赋值默认按当前时间下发，1小时内取件）                                                                                                               
| channelCode          | 渠道编码，仅限ISV商家传值，报文示例"extraInfoList" : [ {"attrName" : "channelCode","attrVal" : "B0101020070191"}]                                                                              
| haDeliveryOrderId    | 医管局方主键，用作与顺丰沟通用途。与医管局沟通订单状态时，首要使用字段。                                                                                                                                           |
| haGoVerifyNum        | 派送时核对客人独有的IDhaDeliveryLabelNums                                                                                                                                                |
| haDeliveryLabelNums  | 揽收时凭核对HA_deliveryLabelNums子单的编号，多个用竖划线分隔，如1234567893-N01&amp;#124;1234567893-D02                                                                                               |
| haOthers             | JOSN格式用作储存参考资料，地区以大数据方式捞取数据，每日产出派送结果报表明细                                                                                                                                       |
| monthlyCustomerId    | 下单月结卡号，用于标识下单客户的月结卡号，不用于结算，应用场景：到付折扣                                                                                                                                           |
| subsidySource        | 国补标识，dy为抖音；sn为苏宁；tb为淘天；pdd为拼多多；ks为快手；wph为唯品会；dw为得物；xhs为小红书；sph为视频号；merchant为商家自营国补; cancel为取消标识；                                                                               |
| infoCollectSource    | 采集服务标识，dy为抖音；sn为苏宁；tb为淘天；pdd为拼多多；ks为快手；wph为唯品会；dw为得物；xhs为小红书；sph为视频号；merchant为商家自营采集服务; cancel为取消标识                                                                            |
| wpMerchantCode       | 微派商户编码，微派会给每个接入客户提供一个唯一编码<br> **测试可用：** B2019020002-0014                                                                                                                 |
| wpServiceCode        | 微派任务编码，生产环境需与微派申请任务编码<br>**测试可用：**<br>Pro24125839--手机<br>Pro24125847--平板<br>Pro24125848--电脑<br>Pro24125849--智能穿戴                                 |
| wpExtJson            | "{"sn":"123456","imei":"123456"}" ---sn为sn编码，只支持一个，imei为手机/平板emei号，只支持一个                                                                                                       |
| realTimeLogistics    | 大同城物流字段专用 (有用到的字段就传递，没有用到就不传) <br>顺手送传值示例：<br>{  "attrVal": "{\\"vehicleScene\\":\\"60\\",\\"fqClientCode\\":\\"shunshousong\\"}", "attrName":"realTimeLogistics"} |

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

###### <a id="commonRespParam">2.4. 公共响应参数 

| # | 属性名       | 类型(约束) | 必填 | 默认值 | 描述                   |
|---|-----------|--------|----|-----|----------------------|
| 1 | success   | String | 是  |     | true 请求成功，false 请求失败 |
| 2 | errorCode | String | 是  |     | 错误编码，S0000成功         |
| 3 | errorMsg  | String | 是  |     | 错误描述                 |
| 4 | msgData   | String | 是  |     | 返回的详细数据              |

###### <a id="respParam">2.5. 响应参数<msgData> 

###### 2.5.1  元素<响应> OrderResponse

| #  | 属性名                       | 类型(约束)      | 必填 | 描述                                                                                                    |
|----|---------------------------|-------------|----|-------------------------------------------------------------------------------------------------------|
| 1  | orderId                   | String(64)  | 是  | 客户订单号                                                                                                 |
| 2  | originCode                | String(10)  | 否  | 原寄地区域代码，可用于顺丰 电子运单标签打印                                                                                |
| 3  | destCode                  | String(10)  | 否  | 目的地区域代码，可用于顺丰 电子运单标签打印                                                                                |
| 4  | filterResult              | Number(2)   | 否  | 筛单结果： 1：人工确认 2：可收派 3：不可以收派                                                                            |
| 5  | remark                    | String(100) | 条件 | 如果filter_result=3时为必填， 不可以收派的原因代码： 1：收方超范围 2：派方超范围 3：其它原因 高峰管控提示信息 【数字】：【高峰管控提示信息】 (如 4：温馨提示 ，1：春运延时) |
| 6  | url                       | Number(200) | 否  | 二维码URL （用于CX退货操作的URL）                                                                                 |
| 7  | paymentLink               | String(200) | 否  | 用于第三方支付运费的URL                                                                                         |
| 8  | isUpstairs                | String(1)   | 否  | 是否送货上楼 1:是                                                                                            |
| 9  | isSpecialWarehouseService | String(4)   | 否  | true 包含特殊仓库增值服务                                                                                       |
| 10 | serviceList               | List        | 否  | 下单补充的增值服务信息                                                                                           |
| 11 | returnExtraInfoList       | List        | 否  | 返回信息扩展属性                                                                                              |
| 12 | waybillNoInfoList         | List        | 否  | 顺丰运单号                                                                                                 |
| 13 | routeLabelInfo            | List        | 是  | 路由标签，除少量特殊场景用户外，其余均会返回                                                                                |
| 14 | scenePlanCode             | String(50)  | 否  | 组件服务编码                                                                                                |

###### 2.5.2  元素<响应> OrderResponse/routeLabelInfo

| # | 属性名            | 类型(约束)         | 必填 | 描述                           |
|---|----------------|----------------|----|------------------------------|
| 1 | code           | String(30)     | 是  | 返回调用结果，1000：调用成功； 其他调用失败     |
| 2 | routeLabelData | routeLabelData | 是  | 路由标签数据详细数据，除少量特殊场景用户外，其余均会返回 |
| 3 | message        | String(1000)   | 否  | 失败异常描述                       |

###### 2.5.3 元素<响应> OrderResponse/routeLabelInfo/routeLabelData

| #  | 属性名                 | 类型(约束)       | 必填 | 描述                                                                                                                                                                                                        |
|----|---------------------|--------------|----|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1  | waybillNo           | String(30)   | 否  | 运单号                                                                                                                                                                                                       |
| 2  | sourceTransferCode  | String(60)   | 否  | 原寄地中转场                                                                                                                                                                                                    |
| 3  | sourceCityCode      | String(60)   | 否  | 原寄地城市代码                                                                                                                                                                                                   |
| 4  | sourceDeptCode      | String(60)   | 否  | 原寄地网点代码                                                                                                                                                                                                   |
| 5  | sourceTeamCode      | String(60    | 否  | 原寄地单元区域                                                                                                                                                                                                   |
| 6  | destCityCode        | String(60)   | 否  | 目的地城市代码, eg:755                                                                                                                                                                                           |
| 7  | destDeptCode        | String(60)   | 否  | 目的地网点代码, eg:755AQ                                                                                                                                                                                         |
| 8  | destDeptCodeMapping | String(60)   | 否  | 目的地网点代码映射码                                                                                                                                                                                                |
| 9  | destTeamCode        | String(60)   | 否  | 目的地单元区域, eg:001                                                                                                                                                                                           |
| 10 | destTeamCodeMapping | String(60)   | 否  | 目的地单元区域映射码                                                                                                                                                                                                |
| 11 | destTransferCode    | String(60)   | 否  | 目的地中转场                                                                                                                                                                                                    |
| 12 | destRouteLabel      | String(200)  | 是  | 若返回路由标签，则此项必会返回。如果手打是一段码，检查是否地址异常。打单时的路由标签信息如果是大网的路由标签,这里的值是目的地网点代码,如果 是同城配的路由标签,这里的值是根据同城配的设置映射出来的值,不同的配置结果会不一样,不能根据-符号切分(如:上海同城配,可能是:集散点-目的地网点-接驳点,也有可能是目的地网点代码-集散点-接驳点)                                |
| 13 | proName             | String(60)   | 否  | 产品名称 对应RLS:pro_name                                                                                                                                                                                       |
| 14 | cargoTypeCode       | String(30)   | 否  | 快件内容: 如:C816、SP601                                                                                                                                                                                        |
| 15 | limitTypeCode       | String(30)   | 否  | 时效代码, 如:T4                                                                                                                                                                                                |
| 16 | expressTypeCode     | String(30)   | 否  | 产品类型,如:B1                                                                                                                                                                                                 |
| 17 | codingMapping       | String(60)   | 是  | 入港映射码 eg:S10 地址详细必会返回                                                                                                                                                                                     |
| 18 | codingMappingOut    | String(60)   | 否  | 出港映射码                                                                                                                                                                                                     |
| 19 | xbFlag              | String(30)   | 否  | XB标志 0:不需要打印XB 1:需要打印XB                                                                                                                                                                                   |
| 20 | printFlag           | String(60)   | 否  | 打印标志 返回值总共有9位,每位只 有0和1两种,0表示按丰密 面单默认的规则,1是显示, 顺序如下,如111110000 表示打印寄方姓名、寄方 电话、寄方公司名、寄方 地址和重量,收方姓名、收 方电话、收方公司和收方 地址按丰密面单默认规则 1:寄方姓名 2:寄方电话 3:寄方公司名 4:寄方地址 5:重量 6:收方姓名 7:收方电话 8:收方公司名 9:收方地址                 |
| 21 | twoDimensionCode    | String(600)  | 否  | 二维码 根据规则生成字符串信息, 格式为MMM={‘k1’:’(目的 地中转场代码)’,‘k2’:’(目的 地原始网点代码)’,‘k3’:’(目 的地单元区域)’,‘k4’:’(附件 通过三维码(express_type_code、 limit_type_code、 cargo_type_code)映射时效类型)’,‘k5’:’(运单 号)’,‘k6’:’(AB标识)’,‘k7’:’( 校验码)’} |
| 22 | proCode             | String(30)   | 否  | 时效类型: 值为二维码中的K4                                                                                                                                                                                           |
| 23 | printIcon           | String(100)  | 否  | 打印图标,根据托寄物判断需 要打印的图标(重货,蟹类,生鲜,易碎，Z标) 返回值有8位，每一位只有0和1两种， 0表示按运单默认的规则， 1表示显示。后面两位默认0备用。 顺序如下：重货,蟹类,生鲜,易碎,医药类,Z标,酒标,0 如：00000000表示不需要打印重货，蟹类，生鲜，易碎 ,医药,Z标,酒标,备用                                              |
| 24 | abFlag              | String(30)   | 否  | AB标                                                                                                                                                                                                       |
| 25 | waybillIconList     | List         | 否  | 面单图标                                                                                                                                                                                                      |
| 25 | errMsg              | String(1000) | 否  | 查询出现异常时返回信息。 返回代码: 0 系统异常 1 未找到面单                                                                                                                                                                         |
| 26 | destPortCode        | String(100)  | 否  | 目的地口岸代码                                                                                                                                                                                                   |
| 27 | destCountry         | String(50)   | 否  | 目的国别(国别代码如:JP)                                                                                                                                                                                            |
| 28 | destPostCode        | String(100)  | 否  | 目的地邮编                                                                                                                                                                                                     |
| 29 | goodsValueTotal     | String(30)   | 否  | 总价值(保留两位小数,数字类型,可补位)                                                                                                                                                                                      |
| 30 | currencySymbol      | String(30)   | 否  | 币种                                                                                                                                                                                                        |
| 31 | goodsNumber         | String(20)   | 否  | 件数                                                                                                                                                                                                        |
| 32 | destAddrKeyWord     | String(100)  | 否  | 目的地地址关键词                                                                                                                                                                                                  |
| 33 | noToDoorPayment     | String(1)    | 否  | 乡村件不上门标签                                                                                                                                                                                                  |

###### <a id="requestJsonPost">2.6. 请求示例\应用场景(JSON)示例 

请求报文:（msgData字段）:

```json
{
  "language": "zh-CN",
  "orderId": "F2_20200604180946",
  "customsInfo": {
    "declaredValue": 6000.9654
  },
  "cargoDetails": [
    {
      "amount": 100.5111,
      "count": 2.365,
      "currency": "HKD",
      "goodPrepardNo": "AAAA002",
      "hsCode": "AAAA004",
      "name": "护肤品1",
      "productRecordNo": "AAAA001",
      "sourceArea": "CHN",
      "taxNo": "AAAA003",
      "unit": "个",
      "weight": 6.1
    }
  ],
  "cargoDesc": "苹果",
  "extraInfoList": [
    {
      "attrName": "attr015",
      "attrVal": "1,2,3,4"
    },
    {
      "attrName": "attr014",
      "attrVal": "singBackInfo"
    },
    {
      "attrName": "hhtWaybill",
      "attrVal": "0"
    },
    {
      "attrName": "shopCode",
      "attrVal": ""
    },
    {
      "attrName": "merchantPayOrderNo",
      "attrVal": "184oe3725c7n3qi3jysa"
    },
    {
      "attrName": "podModelAddress",
      "attrVal": "EOS-FSS-CORE"
    }
  ],
  "serviceList": [
    {
      "name": "INSURE",
      "value": "3000"
    }
  ],
  "contactInfoList": [
    {
      "address": "软件产业基地11栋",
      "city": "深圳市",
      "contact": "顺小丰",
      "contactType": 1,
      "country": "CN",
      "county": "南山区",
      "mobile": "13480155048",
      "postCode": "580058",
      "province": "广东省",
      "tel": "4006789888"
    },
    {
      "address": "广东省广州市白云区湖北大厦",
      "city": "",
      "company": "顺丰速运",
      "contact": "顺小丰",
      "contactType": 2,
      "country": "CN",
      "county": "",
      "mobile": "13925211148",
      "postCode": "580058",
      "province": "",
      "tel": "18688806057"
    }
  ],
  "monthlyCard": "",
  "payMethod": 1,
  "expressTypeId": 1,
  "parcelQty": 4,
  "totalLength": 12.0,
  "totalWidth": 12.0,
  "totalHeight": 10.0,
  "volume": 8.0,
  "totalWeight": 11.1,
  "totalNetWeight": "12.1",
  "sendStartTm": "2020-03-10 10:00:00",
  "isDocall": 1,
  "isSignBack": 1
}
```

香港地址下单请求报文示例:（msgData字段）:

```json
{
  "cargoDetails": [
    {
      "count": 2.365,
      "unit": "个",
      "weight": 6.1,
      "amount": 100.5111,
      "currency": "HKD",
      "name": "护肤品1",
      "sourceArea": "CHN"
    }
  ],
  "contactInfoList": [
    {
      "address": "香港中文大学",
      "city": "香港",
      "company": "顺丰速运",
      "contact": "小丰",
      "contactType": 1,
      "country": "852",
      "postCode": "580058",
      "tel": "18665312825"
    },
    {
      "address": "深圳大学",
      "city": "深圳市",
      "company": "顺丰速运",
      "contact": "小丰",
      "contactType": 2,
      "country": "86",
      "county": "南山区",
      "mobile": "18576651234",
      "postCode": "",
      "province": "广东省",
      "tel": "18576651234",
      "zoneCode": ""
    }
  ],
  "customsInfo": {
    "declaredValue": 80.0,
    "declaredValueCurrency": "USD"
  },
  "language": "CH-zh",
  "monthlyCard": "8521234567",
  "expressTypeId": "276",
  "orderId": "hk-test000000000000001",
  "sendStartTm": "2024-03-15 10:00:00",
  "isDocall": 1
}
```

###### <a id="responseJsonPost">2.7. 返回示例\应用场景(JSON)示例 

响应报文:

- 成功响应:

```json
{
  "apiErrorMsg": "",
  "apiResponseID": "0001727E2494543FEA449BF48D24123F",
  "apiResultCode": "A1000",
  "apiResultData": {
    "success": true,
    "errorCode": "S0000",
    "errorMsg": null,
    "msgData": {
      "orderId": "QIAO-20200528-006",
      "originCode": "719",
      "destCode": "710",
      "filterResult": 2,
      "remark": "",
      "url": null,
      "paymentLink": null,
      "isUpstairs": null,
      "isSpecialWarehouseService": null,
      "mappingMark": null,
      "agentMailno": null,
      "returnExtraInfoList": null,
      "waybillNoInfoList": [
        {
          "waybillType": 1,
          "waybillNo": "SF7444400043266"
        }
      ],
      "routeLabelInfo": [
        {
          "code": "1000",
          "routeLabelData": {
            "waybillNo": "SF7444400043266",
            "sourceTransferCode": "",
            "sourceCityCode": "719",
            "sourceDeptCode": "719",
            "sourceTeamCode": "",
            "destCityCode": "710",
            "destDeptCode": "710F",
            "destDeptCodeMapping": "",
            "destTeamCode": "",
            "destTeamCodeMapping": "",
            "destTransferCode": "710VA",
            "destRouteLabel": "710VA-F",
            "proName": "顺丰标快",
            "cargoTypeCode": "C201",
            "limitTypeCode": "T4",
            "expressTypeCode": "B1",
            "codingMapping": "",
            "codingMappingOut": "",
            "xbFlag": "0",
            "printFlag": "000000000",
            "twoDimensionCode": "MMM={'k1':'710VA','k2':'710F','k3':'','k4':'T4','k5':'SF7444400043266','k6':'','k7':'e177bc63'}",
            "proCode": "T4",
            "printIcon": "00000000",
            "abFlag": "",
            "destPortCode": "",
            "destCountry": "",
            "destPostCode": "",
            "goodsValueTotal": "",
            "currencySymbol": "",
            "cusBatch": "",
            "goodsNumber": "",
            "errMsg": "",
            "checkCode": "e177bc63",
            "proIcon": "",
            "fileIcon": "",
            "fbaIcon": "",
            "icsmIcon": "",
            "destGisDeptCode": "710F",
            "newIcon": null
          }
          ",
          "message": "SF7444400043266:"
        }
      ],
      "contactInfoList": null
    }
  }
}
```

- 失败响应：

```json
{
  "success": false,
  "errorCode": "8114",
  "errorMsg": "传入了不可发货的月结卡号",
  "msgData": null
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
| A1008 | 数据解密失败                                                                                                | 在特殊场景中使用，如有出现请报障人工处理                                                                                                                                                                                                                                           |
| A1009 | 目标服务异常或不可达                                                                                            | 接口下游服务异常，如有出现请报障人工处理                                                                                                                                                                                                                                           |
| A1010 | 状态为沙箱测试                                                                                               | 该问题在老客户中会有此类问题，新客户不会出现，如有出现请报障人工处理                                                                                                                                                                                                                             |
| A1099 | 系统异常                                                                                                  | 接口服务异常，如有出现请报障人工处理                                                                                                                                                                                                                                             |

##### 3.2 业务异常代码

| 错误代码  | 错误中文描述                                              | 错误英文描述                                                           |                          【处理建议】                          |
|-------|-----------------------------------------------------|------------------------------------------------------------------|:--------------------------------------------------------:|
| 1010  | 寄件地址不能为空                                            | Shipper‘s address is required.                                   |                       address不能为空                        |
| 1011  | 寄件联系人不能为空                                           | Shipper‘s contract name is required.                             |                       contact不能为空                        |
| 1012  | 寄件电话不能为空                                            | Shipper‘s telephone number is required.                          |                     mobile和tel不能都为空                      |
| 1014  | 到件地址不能为空                                            | Receiver‘s adress is required.                                   |                       address不能为空                        |
| 1015  | 到件联系人不能为空                                           | Receiver‘s contact name is required.                             |                       contact不能为空                        |
| 1016  | 到件电话不能为空                                            | Receiver‘s telephone number is required.                         |                     mobile和tel不能都为空                      |
| 1020  | 出口件邮编不能为空                                           | Postal code is required for International shipments.             |                       postCode不能为空                       |
| 1023  | 拖寄物品名不能为空                                           | Commodity name is required.                                      |                 cargoDetails下面的name不能为空                  |
| 1028  | 出口件时，拖寄物数量不能为空                                      | Commodity quantity is required for international shipments.      |                 cargoDetails下面的count不能为空                 |
| 1038  | 出口件声明价值不能为空                                         | The declared value is required for International shipments.      |                  cargoDeclaredValue不能为空                  |
| 6126  | 月结卡号不合法                                             | Invalid credit account number.                                   |                 monthlyCard月结卡号必须为10位数字                  |
| 6127  | 增值服务名不能为空                                           | AVS name is required.                                            |                   serviceList下面的name为空                   |
| 6128  | 增值服务名不合法                                            | Invalid AVS name.                                                |                 serviceList 下面name传值不正确                  |
| 6130  | 体积参数不合法                                             | Invalid Volume Parameters                                        |                       volume传参不正确                        |
| 6138  | 代收货款金额传入错误                                          | COD amount data error.                                           |             serviceList中name为COD 对应的value为数字             |
| 6139  | 代收货款金额小于0错误                                         | Error! COD amount is less than 0.                                |            serviceList中name为COD 对应的value必须大于0            |
| 6200  | 国际件寄方邮编不能为空                                         | The shipper postal code is required for International shipment.  |                       postCode不能为空                       |
| 6201  | 国际件到方邮编不能为空                                         | The receiver postal code is required for International shipment. |                       postCode不能为空                       |
| 6202  | 国际件货物数量不能为空                                         | The cargo quantity is required for International shipment.       |                 cargoDetails下面的count不能为空                 |
| 6203  | 国际件货物单位不能为空                                         | The cargo unit is required for International shipment.           |                 cargoDetails下面的unit不能为空                  |
| 6204  | 国际件货物单位重量不能为空                                       | The cargo unit weight is required for International shipment.    |                cargoDetails下面的weight不能为空                 |
| 6205  | 国际件货物单价不能为空                                         | The cargo unit value is required for International shipment.     |                cargoDetails下面的amount不能为空                 |
| 6206  | 国际件货物币种不能为空                                         | The cargo currency is required for International shipment.       |               cargoDetails下面的currency不能为空                |
| 6207  | 国际件原产地不能为空                                          | Origin code is required for International shipment.              |              cargoDetails下面的sourceArea不能为空               |
| 8016  | 重复下单                                                | Duplicated order ID.                                             |                       orderId不能重复                        |
| 8027  | 不存在的业务模板                                            | Business template does not exist.                                |              bizTemplateCode传入了不存在的模板 或者传空了              |
| 8067  | 超过最大能申请子单号数量                                        | Exceed the maximum number of the available sub waybills.         |                   下单接口默认最大申请子单号数量为307个                   |
| 8096  | 您的预约超出今日营业时间，无法上门收件。                                |                                                                  |              sendStartTm传工作时间。或者isDocall传0               |
| 8114  | 传入了不可发货的月结卡号                                        |                                                                  |                    联系销售经理增加该月结卡号下单权限                     |
| 8117  | 下单包裹不能大于307个                                        |                                                                  |                   下单接口默认最大申请子单号数量为307个                   |
| 8119  | 月结卡号不存在或已失效                                         |                                                                  |                  传入的monthlyCard不存在或已失效                   |
| 8194  | 跨境件必须包含申明价值和币别                                      |                                                                  | 跨境件申明价值（declaredValue）和申明价值币别（declaredValueCurrency）必须要传 |
| 8196  | 信息异常                                                |                                                                  |                      收件或者寄件电话号码信息异常                      |
| 8247  | 运单号不合法                                              |                                                                  |         请核实运单号是否是顺丰运单号（注意顺丰生产环境 测试环境 丰桥上面的单不能混用）         |
| 8053  | 目的地不在定时派送服务范围内                                      |                                                                  |              到件地址不支持定时派送。可以去掉定时派送（IN26）增值服务              |
| 8052  | 原寄地不在定时派送服务范围内                                      |                                                                  |              寄件地址不支持定时派送。可以去掉定时派送（IN26）增值服务              |
| 8051  | 定时派送不在时效范围内，下单失败                                    |                                                                  |               传入的时间不在时效范围内，可以根据返回响应的时间段来传值               |
| 8179  | 卡号下未查到关联相应协议                                        |                                                                  |                      需要找销售签订对应的产品协议                      |
| 8177  | 类似 （正值运力高峰期，普通会员（非会员）的寄件通道预约已满，敬请谅解） 提示语组成 BPS：策略编号 |                                                                  |                           高峰管控                           |
| 20012 | 定时派送服务不支持重量超过300KG的快件                               |                                                                  |                   totalWeight不能超过300kg                   |
| 20011 | 产品与定时派送服务时间段不匹配                                     |                                                                  |                 修改TDELIVERY增值服务value1传值                  |
| 8256  | 部分快件产品不支持到付和寄付现结，请调整付款方式后下单                         |                                                                  |           付款方式payMethod传1 3 并且monthlyCard需要传值            |
| 20035 | 托寄物违禁品不可收寄                                          |                                                                  |                  修改托寄物cargoDetails的name                  |
| 20036 | 适用产品不满足                                             |                                                                  |         更改产品expressTypeId重新下单，如不行，请联系顺丰销售业务经理处理          |
| 8057  | 快件类型为空或未配置                                          |                                                                  |              expressTypeId不正确，请参看《快件产品类别表》               |

[速运类接口业务相关错误码](https://open.sf-express.com/developSupport/976720?activeIndex=146623)
 