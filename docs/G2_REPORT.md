# G2 — transport и SCM, промежуточный отчёт

Изменено: scrume-core/authorization, scrume-windows/pipe, service protocol/SCM/CLI; добавлен service-smoke.ps1. Подготовлен DPAPI primitive для последующего G6, без сохранения реальных токенов.

Реализовано: bounded local pipe, explicit DACL, OS token SID/session extraction, pure authorization, sanitized status response, denial of unsupported connect, bounded SCM stop. Native DPAPI protect/unprotect используют machine context + UI_FORBIDDEN, fixed purpose entropy, bounds и zeroizing Rust/native plaintext allocations. **Файлового service-only vault пока нет**; машинный DPAPI сам не запрещает другому пользователю расшифровать доступный ciphertext.

Проверки: fmt/clippy/macOS tests и cross-target Windows checks записываются в docs/evidence/g2-*.txt. 27 Rust tests на macOS, дополнительно8 mock HTTP и4 Python tests;34 signed vectors проверяются отдельно. Три DPAPI tests существуют только для Windows и на macOS не исполняются. Количество фактически выполненных native Windows tests —0 до запуска в guest.

Не проверено: SCM install/start/stop в VM, DACL/anonymous/second-user/remote deny, actual TokenSessionId, timeout/read response behaviour, DPAPI runtime/ACL/boot. PowerShell script не считается проверенным от одного просмотра текста или cross-check.

Безопасность: mutation fail closed, source of identity — Windows token, server не выполняет raw commands, status не содержит SID/token/profile. Перед auth IPC обязательно server authenticity, owner bootstrap и protected storage. RDP/fast switching ограничены, см. ADR0005.

Следующий шаг: Windows guest + build → SCM/status smoke → исправления по actual evidence → identity/security tests. Реальный TUN POC G5 остаётся до GUI/MSI; нужны approved staging и отдельная x64 приёмка.


Дополнение 13.11.2026: status client получил bounded retry только при ERROR_PIPE_BUSY; in/out buffers явно ограничены. Подготовлен opt-in native pipe test (реальная OS identity, duplicate listener, oversize, reconnect, idle stop), который запускается launcher вместе с3 DPAPI tests. Обычный hosted CI пропускает console-only test. SCM smoke теперь использует bounded ServiceController wait и выводит PASS после подтверждённого удаления своей SCM записи. Native выполнение этих дополнений пока NOT_RUN. Evidence followup checks/build — отдельные файлы g2-followup-*.txt.


Runtime 13.11.2026: `run-native-tests.cmd` в Windows11 Pro ARM64 / Fusion — **4 passed,0 failed,0 ignored,0.15s**.3 DPAPI primitive tests и1 same-user console pipe test. Результат наблюдался агентом в окне VM и подтверждён полным логом пользователя. Evidence: docs/evidence/g2-native-arm64-tests.txt. Это не SCM smoke, vault ownership или x64 acceptance.
