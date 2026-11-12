# ADR-0004: подпись точных байтов вместо JCS

Статус: PROPOSAL; реализуется и проверяется локально, сервер ещё не согласован.

JCS требует полностью совпадающей обработки чисел и Unicode у Python/Rust. Для этого небольшого продукта проще signed payload bytes: сервер сериализует JSON один раз, подписывает domain prefix + полученные байты, передаёт payload как base64url без padding. Клиент проверяет подпись исходных decoded bytes и только затем разбирает JSON. Перестановка полей не требует каноникализации, потому что повторной сериализации перед verify нет. Дубликаты полей/unknown fields/неправильные типы отклоняются строгими Rust structs. Время — целые Unix seconds; floating point не нужен.

Алгоритм Ed25519, verify_strict; prefix `scrumevpn/bundle/v1` + NUL; outer key_id и внутренний signed key_id обязаны совпадать. Внутри подписаны context, schema_version, enrollment_id, version, issued_at, expires_at, profile и policy. Менять expiry вне signature невозможно. Keys pinned out-of-band; key rotation требует уже доверенного signed installer/transition, не download key из недоверенного ответа. Anti replay: persistent version+payload hash; равная версия допустима только с теми же байтами. Уменьшение версии запрещено. Trusted clock rollback требует fail closed и admin recovery; первый core verifier требует clock floor как вход.

Это намеренное упрощение wire format v1, а не заявление о совместимости с исходным сервером. Его rollout требует подтверждения владельца issuer. Исходный формат `profile + signature + unsigned expires_at` не поддерживать как fallback.
