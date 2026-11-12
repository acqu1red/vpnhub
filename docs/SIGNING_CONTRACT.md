# Signing contract v1

PROPOSAL, реализован в Rust core и независимых Node public test vectors. Не утверждён backend owner.

Outer JSON: ровно `key_id`, `payload`, `signature`. `key_id` ≤64 bytes. payload/signature base64url RFC4648 URL-safe, без padding. MAX outer65536 bytes, decoded payload32768 bytes. Signature — 64 bytes Ed25519. Embedded trust key — 32 bytes Ed25519; не загружается по URL из envelope.

Подписываются байты `b"scrumevpn/bundle/v1\x00" + base64url_decode(payload)`. Никакой повторной сериализации, сортировки ключей, нормализации Unicode или canonicalization на verifier. Issuer подписывает окончательные UTF-8 JSON bytes один раз и передаёт их неизменными. Python пример алгоритма: `private_key.sign(b"scrumevpn/bundle/v1\x00" + payload_bytes)`; private key только в server secret store.

Внутренний JSON строго типизирован, deny unknown/duplicate fields. Все поля обязательны. Все timestamps/versions — u64 integers. Context=`scrumevpn/bundle/v1`, schema_version=1. key_id должен совпасть с outer. enrollment_id должен совпасть с ранее полученным expected enrollment, не доверять самому payload как источнику ожидаемого id.

Проверки по порядку: bounds → outer parse → known key → base64 decode/bounds → signature strict → typed payload parse → context/key/schema/device binding → trusted clock floor/future skew → hard expiry/max lease → version/digest rollback guard → profile/policy semantic validation. Error codes не включают raw input.

PROPOSAL defaults: issue skew≤120s, lease≤86400s, expiry строго `expires_at>now`. Persistent rollback floor — `(version, SHA256(payload bytes))` для enrollment; equal version допустима только с тем же digest. На новом enrollment старый floor не применяется автоматически: enrollment binding хранится вместе с floor. Key rotation не сбрасывает floor. Floor и bundle записываются атомарно до эффекта apply; восстановление старого backup не должно уменьшать high-water без специального admin recovery.

Clock: wall-clock используется для signed dates, monotonic — scheduling; trusted_time_floor сохраняется service. Core проверяет переданный floor, но persistence и обнаружение системного clock rollback — будущий service adapter. Нельзя утверждать защиту от clock attack только по unit test.

Fixtures `tests/fixtures/signing-v1.json` и `signing-python-v1.json` содержат только public keys/payloads/signatures, без private keys. Node signer и Python reference создают ключи в памяти и сохраняют только public часть; generate запускать намеренно, не на каждом CI. Rust проверяет34 Node vectors и1 Python vector, включая отказ после повторной сериализации тех же данных. Python cryptography50.0.2 также проверил все35 signatures/negative vectors. G3.09 local interoperability выполнена; production issuer integration и contract approval остаются открытыми. См. PYTHON_SIGNING_REFERENCE.md.

Неподдерживаемые функции (kill switch true, nonempty domain exclusions) в первой core реализации отвергаются, а не игнорируются. Это промежуточный feature set, не изменение исходного объёма приёмки.
