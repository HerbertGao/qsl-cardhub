# 优化前端菜单UI布局

## 概述

优化左侧功能菜单的UI样式，使菜单项紧贴侧边栏边缘，提供更清晰的视觉效果和更大的交互区域。

## 变更内容

**修改文件**：`web/src/App.vue`

**CSS 变更**：
```css
/* 修改前 */
.el-menu-item {
  border-radius: 8px;
  margin: 4px 15px;
}

/* 修改后 */
.el-menu-item {
  border-radius: 0 8px 8px 0;
  margin: 4px 0;
}
```

## 视觉效果

- ✅ 菜单项左边缘紧贴侧边栏
- ✅ 菜单项右侧保留圆角
- ✅ 保持上下 4px 间距
- ✅ 激活和悬停状态清晰可见

## 实施时间

约 10-15 分钟

## 文档

- [提案](./proposal.md) - 详细的变更说明和验收标准
- [任务清单](./tasks.md) - 具体的实施步骤
- [规范](./specs/ui-design/spec.md) - UI设计需求规范

## 验证

```bash
openspec-cn validate optimize-menu-ui --strict
```

✅ 验证通过
