; QSL CardHub NSIS 钩子
; 用于自定义安装和卸载行为

; 安装后钩子：重命名快捷方式为中文名称
!macro NSIS_HOOK_POSTINSTALL
  ; 重命名桌面快捷方式
  Rename "$DESKTOP\${PRODUCTNAME}.lnk" "$DESKTOP\QSL 分卡助手.lnk"

  ; 重命名开始菜单快捷方式
  Rename "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$SMPROGRAMS\QSL 分卡助手.lnk"
!macroend

; 卸载前钩子：询问用户是否删除用户数据
!macro NSIS_HOOK_PREUNINSTALL
  ; 询问用户是否删除用户数据
  MessageBox MB_YESNO|MB_ICONQUESTION \
    "是否同时删除所有用户数据？$\n$\n包括：$\n  - 数据库（QSL 卡片记录）$\n  - 配置文件$\n  - 已保存的账号凭据$\n$\n数据目录：$APPDATA\qsl-cardhub" \
    /SD IDNO IDYES deleteUserData IDNO skipDelete

  deleteUserData:
    ; 1. 删除用户数据目录（包含数据库、配置文件、加密凭据文件）
    RMDir /r "$APPDATA\qsl-cardhub"

    ; 2. 清除 Windows 凭据管理器中的凭据
    ; Rust keyring crate 使用 "username.service" 格式作为 target name
    ; 忽略删除失败的情况（凭据可能不存在）

    ; QRZ.cn 凭据
    nsExec::Exec 'cmdkey /delete:"qsl-cardhub:qrz:username.qsl-cardhub"'
    nsExec::Exec 'cmdkey /delete:"qsl-cardhub:qrz:password.qsl-cardhub"'
    nsExec::Exec 'cmdkey /delete:"qsl-cardhub:qrz:session.qsl-cardhub"'

    ; QRZ.com 凭据
    nsExec::Exec 'cmdkey /delete:"qsl-cardhub:qrz.com:username.qsl-cardhub"'
    nsExec::Exec 'cmdkey /delete:"qsl-cardhub:qrz.com:password.qsl-cardhub"'
    nsExec::Exec 'cmdkey /delete:"qsl-cardhub:qrz.com:session.qsl-cardhub"'

    ; 顺丰速运凭据
    nsExec::Exec 'cmdkey /delete:"qsl-cardhub:sf:partner_id.qsl-cardhub"'
    nsExec::Exec 'cmdkey /delete:"qsl-cardhub:sf:checkword_prod.qsl-cardhub"'
    nsExec::Exec 'cmdkey /delete:"qsl-cardhub:sf:checkword_sandbox.qsl-cardhub"'
    nsExec::Exec 'cmdkey /delete:"qsl-cardhub:sf:environment.qsl-cardhub"'

    ; 云同步凭据
    nsExec::Exec 'cmdkey /delete:"qsl-cardhub:sync:api_key.qsl-cardhub"'

    Goto done

  skipDelete:
    ; 用户选择不删除数据，继续正常卸载

  done:
!macroend

; 卸载后钩子：删除中文名称的快捷方式（如果存在）
!macro NSIS_HOOK_POSTUNINSTALL
  ; 删除中文名称的桌面快捷方式
  Delete "$DESKTOP\QSL 分卡助手.lnk"

  ; 删除中文名称的开始菜单快捷方式
  Delete "$SMPROGRAMS\QSL 分卡助手.lnk"
!macroend
