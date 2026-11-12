# Локальная разработка без production

Запуск: `node tools/mock-issuer.mjs`. Только `127.0.0.1:4200`; email `worker@example.invalid`, публичный тестовый OTP `000000`. Письма не отправляются, Xray/server ничего не меняют. Процесс генерирует ephemeral Ed25519 key, выводит только public trust root, хранит mock state в памяти. Остановка процесса удаляет mock enrollments. Это не backend для эксплуатации.

Контракт: request/verify/bundle в `/vpn/v1`. Request —202 без подтверждения существования сотрудника; verify — только mock install token/enrollment id, без стороннего API key; bundle — signed bytes. Повторный verify с тем же enrollment_request_id возвращает прежний результат в течение300s; другой email/code с тем же id —409. Resend≥60s, TTL600s, max5 wrong attempts. Два devices получают разные UUID; повторный fetch сохраняет bundle. На окончании lease создаётся новая version, а не изменяется unsigned expiry.

Администраторского HTTP endpoint в mock нет. Test harness имеет методы revoke/setAutoConnect; они проверяют client contract в автоматических tests, но не доказывают серверный Xray RemoveUser или прекращение active TCP/UDP. Все hostnames профиля — example.invalid, поэтому mock config не является подключаемым профилем.

`node --test tests/mock-issuer.test.mjs` запускает8 реальных loopback HTTP tests. `python3 tools/dev.py test` включает их вместе с20 Rust tests,34 public signed vectors и4 Python integrity tests. GUI/HTTP Rust client и настоящая issuer integration ещё не реализованы.

При production adapter не копировать test OTP/in-memory store/dynamic trust. Использовать фактический issuer auth, защищённое хранение signing key, durable enrollment/outbox и утверждённый контракт. Machine service pins trust out-of-band, не доверяет public key из HTTP response.
