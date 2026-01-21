// 集成测试入口
//
// 运行命令: cargo test --test integration

#[path = "integration/template_layout.rs"]
mod template_layout;

#[path = "integration/end_to_end.rs"]
mod end_to_end;

#[path = "integration/backend_integration.rs"]
mod backend_integration;
