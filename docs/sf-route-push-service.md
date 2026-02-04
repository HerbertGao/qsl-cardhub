#### 路由推送接口-速运类API

######   RoutePushService

------------

###### <a id="apiDesc">1. 功能描述</a>

- 该接口用于当路由信息生产后向客户主动推送要求的顺丰运单路由信息。推送方式为增量推送,对于同一个顺丰运单的同一个路由节点,不重复推送。仅推送通过丰桥接口下单的订单路由信息。

  不管您的下订单是在测试环境，还是在生产环境，当您选择了路由推送接口并填写了接收地址，下单会自动生成测试路由并推送。贵司接口能正常接收并正确响应，则测试成功。

  客户需提供一个符合以下规范的HTTP 或 HTTPS URL,以接收顺丰系统推送的信息。

    - 请求方法为:"json"或"form",客户可从配置选项中选择请求方法。

      \1) 请求方法为"json"时,数据类型为"application/json ;charset=UTF-8";

      \2) 请求方法为"form"时,数据类型为"application/x-www-form-urlencoded; charset=UTF-8 "; 请求参数为"content";数据是以URL编码(字符集为UTF-8)的XML。

    - 请求方式为:HTTP POST方式推送给客户。

    - 请求方法为"form"时,客户通过"content"字段接收到数据后,需要先对其进行URL解码,得到相应的XML;请求方法为"json"时,客户可直接从请求的数据流中得到相应的json。

    - 1.请求方法为form，在客户处理XML信息后,向顺丰系统返回响应XML报文,响应XML报文结果只能为OK/ERR(参见XML报文说明),顺丰系统将重新推送此次交易的所有信息;

    - 2.请求方法为json，客户处理完Json信息后，向顺丰系统返回响应json报文，响应Json报文结果包含【return_code（0000:成功 1000：失败）、return_msg】

    - 沙箱环境推送频率:10min/次,沙箱测试环境每单固定推送两条路由数据

##### 2. 接口定义

###### <a id="commonParam">2.1. 公共参数</a>

| 名称         | 值                                |
| ------------ | --------------------------------- |
| 接口服务代码 | RoutePushService                  |
| 批量交易     | 最多10个WaybillRoute元素          |
| 接口类型     | 推送                              |
| 报文类型     | application/x-www-form-urlencoded |

###### <a id="commonReqParam">2.2. 公共请求参数</a>

###### <a id="reqParam">2.3. 请求参数 </a>/WaybillRoute

| #    | 属性名        | 类型（约束） | 必填 | 默认值 | 描述                                                         |
| ---- | ------------- | ------------ | ---- | ------ | ------------------------------------------------------------ |
|1    | mailno    | String    |     |     | 客户运单号
|2    | acceptAddress    | String    |     |     | 收货地址
|3    | reasonName    | String    |     |     | 异常描述
|4    | orderid    | String    |     |     | 客户订单号
|5    | acceptTime    | String    |     |     | 收货时间
|6    | remark    | String    |     |     | 备注
|7    | opCode    | String    |     |     | [操作码](https://open.sf-express.com/developSupport/734349?activeIndex=589678)
|8    | id    | String    |     |     | ID
|9    | reasonCode    | String    |     |     | 异常编码
|10	|firstStatusCode	|String	|||		一级状态码|
|11	|firstStatusName	|String	|||		一级状态码描述|
|10	|secondaryStatusCode|	String	|||		二级状态码||
|10	|secondaryStatusName	|String		|||	二级状态码描述|

注意事项:

- 1)需要与顺丰商务人员沟通确认路由信息的语言，目前支持中文简体，中文繁体和英文。

- 2)需要与顺丰商务人员沟通确认推送的方式：

    - 标准路由推送，可从顺丰商务人员处获取顺丰标准推送路由节点信息列表;

      -定制路由推送，须与顺丰商务人员沟通，客户可基于顺丰所有路由节点（列表可从顺丰商务人员处获取）定制所需的路由节点及其具体描述与操作码。

- 查看[路由信息操作码](https://open.sf-express.com/developSupport/734349?activeIndex=589678)。

###### <a id="respParam">2.4. 响应参数 </a>

| #    | 元素名      | 类型（约束） | 必填 | 描述       |
| ---- | ----------- | ------------ | ---- | ---------- |
| 1    | return_code | String       | Y    | 返回响应码 |
| 2    | return_msg  | String       | Y    | 返回消息   |


#####  

###### <a id="requestJsonPost">2.5. 请求示例\应用场景(JSON)示例 </a>

请求报文:

```json
{
    "Body": {
        "WaybillRoute": [{
            "mailno": "SF7444400031887",
            "acceptAddress": "深圳市",
            "reasonName": "",
            "orderid": "202003225d33322239ddW1df5t3",
            "acceptTime": "2020-05-11 16:56:54",
            "remark": "顺丰速运 已收取快件",
            "opCode": "50",
            "id": "158918741444476",
            "reasonCode": ""
        },
        {
            "mailno": "SF7444400031887",
            "acceptAddress": "郑州市",
            "reasonName": "",
            "orderid": "202003225d33322239ddW1df5t3",
            "acceptTime": "2020-05-11 16:56:54",
            "remark": "快件到达 【郑州园博中转场】",
            "opCode": "31",
            "id": "158918741457126",
            "reasonCode": ""
        }]
    }
}
```

###### <a id="responseJsonPost">2.6. 返回示例\应用场景(JSON)示例 </a>

响应报文:

- 成功响应:

```json
{
    "return_code": "0000",
    "return_msg": "成功"
}
```

- 失败报文-范例1

```json
{
    "return_code": "1000",
    "return_msg": "系统异常"
}
```

###### <a id="requestJsonPost">2.7. 请求示例\应用场景(XML)示例 </a>

请求报文:

```xml
<?xml version='1.0' encoding='UTF-8'?>
<Request service="RoutePushService" lang="zh-CN">
    <Body>
        <WaybillRoute mailno="SF7444400031887" acceptAddress="深圳市" reasonName="" orderid="202003225d33322239ddW1df5t3" acceptTime="2020-05-11 16:56:54" remark="顺丰速运 已收取快件" opCode="50" id="158918741444476" reasonCode=""/>
        <WaybillRoute mailno="SF7444400031887" acceptAddress="郑州市" reasonName="" orderid="202003225d33322239ddW1df5t3" acceptTime="2020-05-11 16:56:54" remark="快件到达 【郑州园博中转场】" opCode="31" id="158918741457126" reasonCode=""/>
    </Body>
</Request>
```

###### <a id="responseJsonPost">2.8. 返回示例\应用场景(XML)示例 </a>

响应报文:

- 成功响应:

```xml
<Response service="RoutePushService">
<Head>OK</Head>
</Response>
```

- 失败报文-范例1

```xml
<Response service="RoutePushService">
<Head>ERR</Head>
<ERROR code="4001">系统发生数据错误或运行时异常</ERROR>
</Response>
```

###### <a id="errorCode">3.1. 错误代码 </a>

##### 3.1 （API）平台结果代码列表

| 标识  | 说明                                                         | 解决方法  |
| ----- | ------------------------------------------------------------ | ------|
| A1000 | 统一接入平台校验成功，调用后端服务成功;<br>注意：<br>不代表后端业务处理成功，实际业务处理结果，<br>需要查看响应属性apiResultData中的详细结果 |表示接口调用正常|
| A1001 | 必传参数不可为空                                             |请做以下几点检查：<br>1、参数列表必传字段未填写<br>2、请求报文头，未配置Content-type：application/x-www-form-urlencoded <br>3、参数key存在空格问题<br>4、http请求参数都通过http URL编码传送<br>5、业务数据报文（msgData）为json报文数据格式<br>6、接口整体报文为form表单|
| A1011 | OAuth2认证失败                                 |使用OAuth2认证会产生该提示<br>请检查业务接口的accessToken参数是否超过2小时，2小时口令会更新，请调用OAuth2认证接口重新获取|
| A1003 | IP无效                                                       |顾客编码（partnerID）配置了需校验IP，请解除校验或按绑定IP调用接口|
| A1004 | 无对应服务权限                                               |可能存在的原因：<br>1、顾客编码（partnerID）没有配置（关联）对应接口的业务接口，请在【开发者对接】-【API列表】中关联；<br>2、接口请求数据与实际环境不一致，请先查看【开发者对接】-【API列表】接口状态：<br>a、【测试中】请使用沙箱环境<br>b、【已上线】请使用正式环境；<br>3、后台配置没有生效，可等待2分钟后在试试，如果还是无法操作，请报障人工处理|
| A1005 | 流量受控                                                     |丰桥为接口功能联调环境，接口整体都有限流管控，单客户编码对应的每个接口限流规则：<br>1、单接口调用30次/s 2、单接口调用3000次/天<br>请尽量只操作功能联调，切勿进行接口压测，谢谢！|
| A1006 | 数字签名无效                                                 |请做以下几点检查：<br>1、确认checkword是否配置正确<br>2、确认verifyCode、msgDigest是否加密加签正确<br>3、确认参数是否有特殊字符，如：&<br>4、参数整体式form表单格式<br>5、非java类语言，需注意特殊字符，目前支持的特殊字符“*”，“空格”，“-”请优先使用<br>6、如果数字签名操作不便，可改为OAuth2认证，Token交互即可，具体参见【开发规范】【鉴权方式说明】|
| A1007 | 重复请求                                                     |接口层暂未启用，业务层主要是针对下单接口，客户请求参数msgData中的orderId请不要重复使用，修改后在调用接口|
| A1008 | 数据解密失败                                                 |在特殊场景中使用，如有出现请报障人工处理|
| A1009 | 目标服务异常或不可达                                         |接口下游服务异常，如有出现请报障人工处理|
| A1010 | 状态为沙箱测试                                               |该问题在老客户中会有此类问题，新客户不会出现，如有出现请报障人工处理|
| A1099 | 系统异常                                                     |接口服务异常，如有出现请报障人工处理|

##### 3.2 业务异常代码

| 原因代码 errorCode | 描述 errorMsg                | 分类     |
| ------------------ | ---------------------------- | -------- |
| S0000              | 成功                         | 是       |
| S0001              | 非法的JSON格式               | 系统错误 |
| S0002              | 必填参数为空                 | 系统错误 |
| S0003              | 系统发生数据错误或运行时异常 | 系统错误 |