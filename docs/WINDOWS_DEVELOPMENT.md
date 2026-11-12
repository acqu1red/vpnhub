# Работа напрямую на Windows

## Восстановить проект

Основной путь после публикации D014 — clone PRIVATE GitHub repository под своей
учётной записью с доступом:

```powershell
git clone https://github.com/acqu1red/vpnhub.git C:\Dev\vpnhub
Set-Location C:\Dev\vpnhub
```

Ниже сохранён offline вариант через ZIP/bundle. После любого варианта продолжить
с CONTINUE_HERE.md; история чата не обязательна.

На исходном компьютере committed source export создаётся командой
`python3 tools/dev.py export-handoff`. Скопировать локальный ZIP и соседний
`.zip.sha256` на Windows. Распаковать, проверить SHA256 архива (`Get-FileHash`),
затем восстановить Git history из `ScrumeVPN/repository.bundle`:

```powershell
git clone C:\Transfer\ScrumeVPN\repository.bundle C:\Dev\ScrumeVPN
Set-Location C:\Dev\ScrumeVPN
git log -5 --oneline
git status --short
```

В архивном HANDOFF-MANIFEST.json записаны HEAD и hashes файлов. После clone HEAD
должен совпасть. `origin` будет указывать на локальный bundle, а не на GitHub;
при необходимости убрать его `git remote remove origin`. Ничего не публиковать
автоматически. Исходная распаковка содержит и рабочие файлы для просмотра без Git.

Прочитать CONTINUE_HERE.md и docs/SESSION_HANDOFF.md. Repo root — ScrumeVPN,
внутри него Cargo.toml и AGENTS.md. Старый внешний SPEC.md уже сохранён как
SPEC_ORIGINAL.md внутри проекта, его содержимое не теряется.

## Инструменты

Нужны Git, Python3.9+ с launcher `py`, Node24.11.0, Rustup и Microsoft C++ Build
Tools/Windows SDK. Выбрать C++ tools для **x64** и, если Windows ARM64, также ARM64.
Установка prerequisites может требовать administrator и принятия лицензии;
это выполняется пользователем штатными установщиками.

Официальные инструкции: [Rust Windows/MSVC](https://rust-lang.org/tools/install/),
[Rustup MSVC prerequisites](https://github.com/rust-lang/rustup/blob/main/doc/user-guide/src/installation/windows-msvc.md),
[Microsoft Build Tools](https://visualstudio.microsoft.com/downloads/),
[Node24.11.0](https://nodejs.org/dist/v24.11.0/), [Python Windows](https://www.python.org/downloads/windows/),
[Git Windows](https://git-scm.com/downloads/win).

Из обычного Windows terminal в repo root:

```powershell
rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy
rustup target add --toolchain 1.99.1 x86_64-pc-windows-msvc
py -3 tools/dev.py doctor
py -3 tools/dev.py test
py -3 tools/dev.py licenses
```

Для ARM64 добавить `aarch64-pc-windows-msvc`. `tools/bootstrap.py` автоматически
скачивает rustup-init только для ранее проверенного Mac ARM64; на новой Windows
использовать официальный Rustup. Не переносить Mac .tools или linker wrapper.
dev.py поддерживает как системный Rustup, так и локальный rustup.exe в .tools.
Line endings заданы .gitattributes; CMD в готовом payload собирается с CRLF.

## Нативная сборка

```powershell
# Windows x64 / целевая архитектура продукта
py -3 tools/build-windows.py --target x86_64-pc-windows-msvc

# Fusion / Windows ARM64: отдельный laboratory target
py -3 tools/build-windows.py --target aarch64-pc-windows-msvc
```

SDK/linker должны поддерживать выбранный target. Это реальная MSVC сборка; на
текущем Mac этот script runtime ещё не выполнялся. Результаты —
`artifacts/windows-lab/`, manifest с actual hashes, `ScrumeVPNSvc.exe`, adapter
tests, `ScrumeVPNLab.exe`, launcher, notices. Никакой установки/смены сети от сборки.
Новый runner hash-bound к service binary; компилировать его этим builder.

Обычный console user: `artifacts\windows-lab\run-native-tests.cmd`.
В disposable Windows VM, **elevated terminal**, после clean snapshot:

```powershell
artifacts\windows-lab\ScrumeVPNLab.exe --disposable-lab
```

На mounted CD путь `D:\ScrumeVPNLab.exe --disposable-lab`. Runner требует elevation,
ничего сам не повышает, не заменяет существующую службу/папку. Service временная,
Manual LocalSystem, fixed protected install path; networking отсутствует.
Сохраняет Program Files lab files для inspection. Повторный чистый прогон — из
snapshot. Ошибку и cleanup проверять до следующего эксперимента.

PowerShell `.ps1` smoke на текущей VM запрещён ExecutionPolicy. Native runner
использует Win32 SCM напрямую, PowerShell не запускает и policies не меняет.
Если приложение блокирует Windows application control/Defender/SmartScreen,
остановиться и записать конкретную ошибку; не ослаблять защиту ради теста.

## Что сохранять при дальнейшем развитии

После этапа: код/lockfiles, STATUS/SESSION_HANDOFF/TRACEABILITY, новые решения в
DECISIONS, redacted evidence с platform/build hash/result, маленький Git commit.
Для каждого claimed PASS указать scope: macOS unit, Windows ARM64, Windows x64,
staging integration. Старые build manifests — историческое evidence, их не
переписывать на PASS после теста; runtime report хранить отдельно.

Игнорируемые .tools/.cache/target/artifacts — восстанавливаемые зависимости и
build outputs, не единственные копии исходников. Важные результаты переносить
в docs/evidence до экспорта. VM и её credential backup — отдельно у пользователя.
На новой Windows потребуются новые device enrollment/secrets, когда реальный
vault/auth будет реализован. Не переносить DPAPI plaintext в ZIP.
