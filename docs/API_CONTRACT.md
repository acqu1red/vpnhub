# API v1 — предложение, не существующий backend

Статус PROPOSAL. Серверный исходник не предоставлен; обращения к production не выполнялись. Namespace `/vpn/v1` — минимальный adapter к существующему issuer. Legacy `/request`, `/verify`, `/app-key`, `/revoke` остаются без несовместимых изменений.

## Клиентские операции

| Метод/URL | Request | Success | Отказ |
|---|---|---|---|
| POST /vpn/v1/auth/request | `{email}` | 202 `{ok:true,retry_after:60}` | 400 invalid; 429 rate; 503 mail/provider unavailable |
| POST /vpn/v1/auth/verify | `{email,code,enrollment_request_id}` | 200 `{install_token,enrollment_id}` | 400 invalid; 401 invalid/expired OTP; 429 attempts; 409 idempotency conflict |
| GET /vpn/v1/bundle | `X-Install-Token`, optional `If-None-Match` | 200 SignedEnvelope; 304 same bytes only | 401 invalid/revoked; 403 disabled; 429; 5xx |

Пример email: `worker@example.invalid`; примеры токенов не сохраняются в репозитории. OTP — ровно 6 ASCII цифр, TTL600/resend60/max5; состояние и проверка лимитов серверные. Запрос email не должен выдавать существование сотрудника; это изменение UX относительно старого 404 требует согласования. Enrollment request UUID выдаётся для одной операции; retry возвращает тот же enrollment при авторизованном/проверенном challenge, а не создаёт новое устройство. Idempotency result с token — sensitive server state с ограниченным TTL/ACL.

Token только в HTTPS header, не в URL/query/logs. Responses `Cache-Control: no-store`; CDN/proxy не кеширует авторизованные ответы. 304 не продлевает signed expiry; если lease заканчивается, сервер отдаёт новый bundle с новой version. Служба не делает `/app-key` calls, если согласован proposed bundle auth; legacy session token не переиспользуется без контракта.

## Поведение HTTP adapter

- Bootstrap issuer HTTPS origin закреплён IT при сборке/установке; GUI/server profile не выбирают URL. Redirect отключён. TLS chain/hostname verification обязательны.
- Connect timeout5s, request15s, max body64KiB — PROPOSAL; retry only safe/idempotent ops. Не повторять OTP verify автоматически с новым request id.
- 401/403 не retry как transient: остановить tunnel/recovery и потребовать auth согласно коду. 429 уважает bounded Retry-After и jitter; 5xx/offline exponential1s..5min. TTL не продлевается из unsigned headers.
- JSON errors имеют `error.code` и `request_id`, не отражают token/OTP или raw upstream exceptions. Service logs не пишут HTTP body/headers.

## Admin/server integration

Revoke — server-internal операция с авторизацией администратора, не командой сотрудника. Сначала durable revoked state, затем outbox RemoveUser, retry/reconcile до фактического запрета. Не считать HTTP200 доказательством прекращения active TCP/UDP. Тестировать active и new connections по отдельности. Пока не доказано — SLA немедленного отзыва OPEN.

UUID/alias привязан к enrollment, не email. Dynamic Xray state не durable: restart recovery должен восстановить desired enabled set. Unique constraint/transaction защищают concurrent enrollments. Существующие серверы/клиенты не перезапускаются ради unit tests.

## Что должно быть подтверждено до staging

Фактические issuer source/version, auth owner, approved endpoints, compatibility consumers, signing key management, registry schema/migration, Xray HandlerService version и limits. Для этого нужен отдельный sanitized source snapshot, не пароли в чате. Локальный mock может реализовывать предложенный контракт до подтверждения.
