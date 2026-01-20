<template>
  <div class="page-content">
    <h1>关于 QSL-CardHub</h1>

    <el-card style="margin-top: 30px; max-width: 600px" shadow="hover">
      <el-descriptions :column="1" border>
        <el-descriptions-item label="应用名称">QSL-CardHub</el-descriptions-item>
        <el-descriptions-item label="版本">v1.0.0</el-descriptions-item>
        <el-descriptions-item label="描述">业余无线电卡片打印系统</el-descriptions-item>
        <el-descriptions-item label="技术栈">
          Rust + Tauri 2 + Vue 3 + Element Plus
        </el-descriptions-item>
        <el-descriptions-item label="平台支持">
          Windows / macOS / Linux
        </el-descriptions-item>
        <el-descriptions-item label="版权">
          © 2026 Herbert Software
        </el-descriptions-item>
      </el-descriptions>
    </el-card>
  </div>
</template>

<script setup>
</script>
