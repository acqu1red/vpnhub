# Python ↔ Rust signing reference — G3.09

PROPOSAL wire format v1, локальная интероперабельность, не патч существующего issuer.

`tools/issuer_signing_reference.py` использует PyCA cryptography50.0.2 Ed25519: подписывает prefix + окончательные UTF-8 JSON bytes, отправляет base64url без padding. Ключ создаётся в памяти, private serialization отсутствует. Fixture сохраняет public key/payload/signature. Backend key storage/rotation/loading остаются отдельной задачей владельца issuer.

Python намеренно сериализует JSON с другим порядком ключей и отступами, чем Node. Rust принимает исходные Python bytes, а повторную сериализацию того же объекта без новой подписи отклоняет. Python также проверил Node signature. Это показывает совместимость предложенного exact-byte формата между тремя реализациями; не доказывает совместимость с фактическим существующим сервером без его контракта.

Воспроизведение генерации на текущем Mac arm64 / CPython3.9:

```sh
python3 -m venv .tools/signing-venv
.tools/signing-venv/bin/python -m pip install --require-hashes -r tools/signing-requirements-macos-arm64-py39.txt
.tools/signing-venv/bin/python tools/issuer_signing_reference.py --generate-public-fixture
.tools/signing-venv/bin/python tools/issuer_signing_reference.py --verify-public-fixtures
python3 tools/dev.py test
```

Lock содержит hashes конкретных Mac wheels, **не** production backend requirements. Все4 downloaded wheels сверены с hashes официального PyPI; version/license/license-path evidence в docs/evidence/python-signing-deps.json. Библиотеки находятся только в optional local dev environment и не включаются в Windows клиент.

Обычный CI не генерирует ключи и не устанавливает Python crypto package: Rust проверяет сохранённый публичный fixture. Перегенерация намеренная и меняет public key/signature; production private key никогда не используется в tests.

Источник API: [PyCA Ed25519](https://cryptography.io/en/latest/hazmat/primitives/asymmetric/ed25519/).
