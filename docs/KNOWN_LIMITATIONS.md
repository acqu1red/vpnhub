# Текущие ограничения

Это начальный инженерный baseline, не готовый VPN, не release candidate и не MSI.

- GUI/Tauri/tray/autostart GUI ещё отсутствуют.
- SCM lifecycle, native status pipe/DACL/token SID/session adapter и DPAPI primitive написаны. Cross-check прошёл для Windows x64/ARM64, лабораторные ARM64 EXE собраны. Native ACL/SCM/DPAPI tests пока не исполнены; файловый vault, supervisor и TUN controller ещё отсутствуют.
- Core boot policy/FSM проверены на macOS. Реальный pre-login boot-connect не проверен и пока не работает как продукт.
- Signing envelope — proposed v1, не подтверждён существующим issuer. Публичные Python/Node signatures проверены Rust; Python также проверил35 public vectors. Реальная server interoperability ещё не проведена.
- Rollback clock/version floor принимается verifier как вход; durable anti rollback/atomic persistence ещё нет.
- Domain excludes и kill_switch=true отвергаются как unsupported. Это временная защита от молчаливого игнорирования; финальный scope требует решения.
- Mock issuer — loopback, example.invalid emails, public test OTP000000, ephemeral keys/in-memory state; нет SMTP, Xray API, durable database и production rate-limit security. Его HTTP допускает обычную JSON.parse semantics, production input validation проектируется отдельно.
- Xray/Wintun binaries скачаны и hash-verified, не запущены на Mac. Настоящие TUN/DNS/routes/IPv6/revoke/20-user tests не проведены.
- Fusion на Mac arm64 не заменяет Windows x64 E2E. Создана ScrumeVPN Lab и установлена Windows11 Pro ARM; VMware Tools установлены; после перезагрузки применяются Windows updates. Точный статус в STATUS.md.
- Нет private remote проекта, выполненного GitHub workflow, code-signing сертификата или существующего issuer source/staging access.
- Development SBOM относится ко всем Cargo targets; окончательный MSI SBOM/licenses/signatures ещё не существуют.

Никаких изменений production issuer/Xray/employees и сети хостового Mac не выполнялось.
