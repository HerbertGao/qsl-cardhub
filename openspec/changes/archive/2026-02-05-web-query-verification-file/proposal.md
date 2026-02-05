## 为什么

在部署 web_query_service 到生产环境时，需要在根目录放置一个验证文件，用于域名所有权验证或第三方服务集成认证。

## 变更内容

- **新增**：在 web_query_service 的静态资源目录中添加验证文件
- 文件名：`e117c5d282297a44d17e18afb8c357f3.txt`
- 文件内容：`b0455d9c5df12acae5ee0d23c065000c5479bb16`
- 文件将随构建自动部署到生产环境根目录

## 功能 (Capabilities)

### 新增功能

- `site-verification`: 站点验证文件生成，确保部署时验证文件存在于根目录

### 修改功能

<!-- 无需修改现有规范 -->

## 影响

### 代码

- `web_query_service/static/` - 添加验证文件

### 部署

- 验证文件将通过 Vite 的 `publicDir` 机制自动复制到构建输出目录
- 部署后可通过 `https://<domain>/e117c5d282297a44d17e18afb8c357f3.txt` 访问
