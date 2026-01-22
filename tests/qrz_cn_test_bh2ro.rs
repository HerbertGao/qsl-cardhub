use qsl_cardhub::qrz::QRZCnClient;
use std::env;

#[tokio::test]
#[ignore]
async fn test_query_bh2ro() {
    // 初始化 env_logger 以便调试
    let _ = env_logger::try_init();

    let username = env::var("QRZ_TEST_USERNAME").expect("需要设置 QRZ_TEST_USERNAME");
    let password = env::var("QRZ_TEST_PASSWORD").expect("需要设置 QRZ_TEST_PASSWORD");

    let client = QRZCnClient::new().expect("创建客户端失败");

    println!("正在登录...");
    client.login(&username, &password).await.expect("登录失败");
    println!("✓ 登录成功\n");

    // 测试 BH2RO
    println!("正在查询 BH2RO...");
    match client.query_callsign("BH2RO").await {
        Ok(Some(info)) => {
            println!("✓ 找到 BH2RO 的地址信息:");
            println!("  呼号: {}", info.callsign);
            println!("  数据来源: {}", info.source);
            if let Some(chinese) = &info.chinese_address {
                println!("  中文地址: {}", chinese);
            }
            if let Some(english) = &info.english_address {
                println!("  英文地址: {}", english);
            }
            if let Some(updated) = &info.updated_at {
                println!("  更新时间: {}", updated);
            }
        }
        Ok(None) => {
            println!("⚠ 未找到 BH2RO（呼号可能不存在）");
        }
        Err(e) => {
            println!("✗ 查询失败: {}", e);
            panic!("查询应该成功");
        }
    }
}
