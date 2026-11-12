# Реестр решений — актуальная версия

Дата: 12.11.2026. Решение основано на прямом указании пользователя; инженерное предложение не становится решением от наличия кода. Полномочия и основания: DECISIONS.md.

| ID | Вопрос | Статус | Основание / следующий шаг |
|---|---|---|---|
| OPEN-01 | OTP или browser auth | OPEN | Mock OTP; production endpoint/provider не подтверждён |
| OPEN-02 | Boot и владение секретом | DECIDED | SOURCE D002–D005: pre-login после initial auth; service-only vault, machine DPAPI+ACL |
| OPEN-03 | Kill switch | OPEN | Unsupported=true пока нет WFP implementation/acceptance |
| OPEN-04 | LAN bypass CIDR | OPEN | Только утверждённый signed список; без автоматического private ranges |
| OPEN-05 | API key из legacy verify | OPEN | PROPOSAL separate VPN adapter; старый endpoint не вызывать |
| OPEN-06 | Источник policy / push | OPEN | PROPOSAL one signed bundle + polling; backend approval нужен |
| OPEN-07 | Signing и rotation | OPEN | PROPOSAL exact-byte signed envelope ADR-0004; local verifier implemented |
| OPEN-08 | Domain split / IPv6 / DNS | OPEN | Domain nonempty reject; dual stack/block требует Windows POC |
| OPEN-09 | Multi-user PC | OPEN | PROPOSAL single owner; local privileged network глобальна |
| OPEN-10 | Signing certificate/process | OPEN | Unsigned QA only до сертификата/approval |
| OPEN-11 | Windows10 edition/patch/ESU | OPEN | Windows10 остаётся исходным target, поддержка требует конкретизации |
| OPEN-12 | Windows и staging | PARTIAL | SOURCE использовать Fusion; VERIFIED Windows11 Pro ARM установлена, Tools установлены, runtime tests pending; x64 acceptance и staging не предоставлены |
| OPEN-13 | Enrollment/import link | DEFERRED | Нет произвольного импорта в GUI; отдельное будущее требование |
| OPEN-14 | SLO 20 users / revoke | OPEN | Измерить active connections, egress, CPU/RAM; threshold подтвердить |
| OPEN-15 | App-key для VPN | OPEN | PROPOSAL install token directly; legacy as session не используется без контракта |
| OPEN-16 | Signed expiry/metadata | OPEN | Локально защищены; server rollout контракта pending |
| OPEN-17 | Exact Xray/TUN/Wintun | PARTIAL | VERIFIED pinned v26.11.9, official Wintun0.14.1 ZIP license/hash; Windows POC NOT_RUN |
| OPEN-18 | Server restart registry | OPEN | PROPOSAL durable registry/outbox/reconcile; issuer source нет |
| OPEN-19 | auto_connect false для уже активного tunnel | OPEN | Boot-disable утверждён; PROPOSAL stop pending auto, active session отдельно от auto_connect |
| OPEN-20 | Backend source + staging change scope | OPEN | Нужны sanitized source/approved test endpoints; production changes не разрешены |

Owner product choices: пользователь/заказчик; implementation proposals: исполнитель; server approval: владелец issuer/IT. Dates/evidence для решённых пунктов в DECISIONS.md; все OPEN записи не имеют согласованной даты решения. OPEN блокирует только зависящие операции. Секреты не запрашивать в чате.
