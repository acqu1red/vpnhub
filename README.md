# ScrumeVPN / vpnhub

Корпоративный VPN-клиент Windows: исходники, исполнимое ТЗ, roadmap и сохранённый
контекст разработки. **Текущая версия — инженерный прототип; готового VPN/MSI пока нет.**

Для продолжения работы прочитать **[CONTINUE_HERE.md](CONTINUE_HERE.md)** и
[точку продолжения](docs/SESSION_HANDOFF.md). Полное состояние —
[STATUS](docs/STATUS.md), установка среды — [Windows development](docs/WINDOWS_DEVELOPMENT.md).

## Начать на Windows

```powershell
git clone https://github.com/acqu1red/vpnhub.git C:\Dev\vpnhub
Set-Location C:\Dev\vpnhub
py -3 tools/dev.py doctor
py -3 tools/dev.py test
```

Нужны Git, Python3.9+, Node24.11.0, Rust1.99.1 и Microsoft C++ Build Tools/Windows
SDK для MSVC сборки. Подробные команды и отдельные x64/ARM64 targets — в инструкции.
Сборка native lab: `py -3 tools/build-windows.py --target x86_64-pc-windows-msvc`.

## Что есть

- Rust core: проверка подписанных bundles, policy, lifecycle и bounded IPC.
- Windows SCM/status pipe, OS caller identity, machine DPAPI primitive.
- Mock issuer, public Node/Python signing vectors, автоматические проверки.
- ТЗ, аудит,13 gates/140 подзадач, архитектура/ADR, evidence и handoff.
- Последние локальные проверки:44 tests PASS; отдельно4 native ARM64 adapter
  tests PASS. Native SCM runner собран, но его runtime ещё не проверен.

Следующий этап — SCM smoke на Windows; далее vault ACL и настоящий TUN на
разрешённом staging. GUI, MSI и production release ещё не реализованы.

## Документы и структура

[SPEC.md](SPEC.md) — копия исходного `vpn/SPEC.md`, совпадает с
[SPEC_ORIGINAL.md](SPEC_ORIGINAL.md). [CODEX_MASTER_SPEC.md](CODEX_MASTER_SPEC.md) —
переработанное ТЗ. [Roadmap](docs/EXECUTION_PLAN.md), [решения](docs/DECISIONS.md),
[приёмка](docs/ACCEPTANCE_REPORT.json), [лицензии](docs/UPSTREAMS_AND_LICENSES.md).

Корень GitHub соответствует рабочей папке `vpn/ScrumeVPN`: Cargo.toml, apps,
crates, tools и docs находятся сразу в корне, без вложенного Git-репозитория.
Исходный внешний SPEC также включён. Существующая история разработки сохранена.
`.tools`, `.cache`, `target`, generated artifacts, VM и credentials не включены
в Git; зависимости/build outputs восстанавливаются из зафиксированных версий.

GitHub Actions проверяет Linux/macOS/Windows build/core scope. Такой CI не
доказывает работу SCM/TUN/MSI. Публикация исходников в GitHub не разворачивает VPN
на production сервере. Этот приватный репозиторий содержит инфраструктурные
сведения исходной документации; не менять его видимость автоматически.
