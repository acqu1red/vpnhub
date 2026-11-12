# Трассируемость требований v2

SOURCE — SPEC_ORIGINAL.md или прямые D002–D005. PROPOSAL — инженерный вариант. VERIFIED относится только к указанному scope; unit и cargo check не означают Windows E2E.

| Требование | Источник | Реализация / план | Приёмка / evidence | Состояние |
|---|---|---|---|---|
| R01 privilege split | SPEC§3 | apps/service SCM scaffold; Tauri позже | A03/A25; Windows cross-check | Compile-only; SYSTEM runtime NOT_RUN |
| R02 one button/tray/brand | SPEC§1/6 | G10 desktop | A05/A06 | NOT_IMPLEMENTED |
| R03 no settings/import | SPEC§5 | IPC allowlist; G10 capabilities | A60 | Core parser verified, GUI pending |
| R04 enrollment per device | SPEC§4 | tools/mock-issuer.mjs; real adapter G7 | A08–15; tests/mock-issuer.test.mjs | VERIFIED mock only |
| R05 boot connect | D002/D003/D005 | Machine.boot(); service boot G6 | no_boot_before_initial_enrollment; signed_policy_can_disable_boot_connect; A07 | VERIFIED core; Windows boot NOT_RUN |
| R06 service-only vault | D004 | ADR0002; scrume-windows/dpapi; G6 | A21/A47; native DPAPI3 tests PASS ARM64 | Machine crypto primitive verified; filesystem vault/ACL NOT_IMPLEMENTED |
| R07 signed profile/policy | SPEC§4/5 | bundle::verify exact bytes; Python reference | A16–20; Node34 + Python1 public vectors | VERIFIED Rust/Node/Python locally; real backend pending |
| R08 binding/anti rollback | PROPOSAL ADR0004 | VerifyContext; VersionFloor | wrong key/device/context/expiry/rollback/version conflict | VERIFIED memory-level; persistent floor pending |
| R09 secure IPC | SPEC§3 | ipc.rs; authorization.rs; native pipe/token adapter; ADR0005 | parser/authorization/response tests; opt-in native console transport test prepared; A22–24 | Pure core + native same-user console transport PASS ARM64; second-user/remote/ACL acceptance NOT_RUN |
| R10 VLESS REALITY TUN | SPEC§7 | pinned Xray; G5/G6 | A26–29 | Download+SHA verified; TUN NOT_RUN |
| R11 DNS/MTU/split | SPEC§4 | typed Profile validation; G8 controller | A30/A32–34 | Validation only; network NOT_RUN |
| R12 IPv6 safety | PROPOSAL | dual-stack/block design G5/G8 | A31 | NOT_RUN |
| R13 rollback/crash | SPEC§3/5 + PROPOSAL | FSM StopAndRecover; native controller G6 | stale health/expiry/revoke/crash core tests; A35–39 | Core verified; actual route recovery NOT_IMPLEMENTED |
| R14 device revoke | SPEC§4 | Mock revoke isolated; server G7 | mock two devices; A41/42/54 | Mock only; active Xray session revoke NOT_RUN |
| R15 restart reconciliation | PROPOSAL | durable registry/outbox G7 | A43 | DESIGN_ONLY |
| R16 server policy | SPEC§5 + D005 | signed policy model; mock version update | mock admin policy test; A44/45 | Core/mock verified; polling/apply pending |
| R17 kill switch | OPEN-03 | unsupported true reject; G9 | unsupported_kill_switch vector; leak tests | NOT_IMPLEMENTED; scope OPEN |
| R18 MSI lifecycle | SPEC§6/8 | G10 WiX | A01/02/04/48–52 | NOT_IMPLEMENTED |
| R19 supply chain/handover | SPEC§2/7 | Cargo.lock/upstreams.lock/dev tools/workflow | dependency inventory/notices/SBOM; A55/57 | Local verified; Windows CI/release audit pending |
| R20 15–20 users | Context (unverified prior discussion) | G7/G12 load plan | A15/53/54 | NOT_RUN; thresholds OPEN |
| R21 Win10/11 x64 | SPEC header | cross target + G5/G11 | A58 | x64 cargo check verified; real OS compatibility NOT_RUN |
| R22 safe errors/logs | PROPOSAL | codes only; VerifiedBundle redacted Debug | redacted_debug test; A47 | Partial; service logger/rotation pending |
| R23 multi-user ownership | PROPOSAL/OPEN-09 | owner SID design G2/G6 | A22/23 + account switch | DESIGN_ONLY |
| R24 evidence/status honesty | AGENTS + current request | STATUS/ACCEPTANCE_REPORT/evidence | distinct scopes/results | Current local report supplied |
| D012/D013 сохранение контекста и Windows handoff | Прямой запрос13.11.2026 | CONTINUE_HERE, SESSION_HANDOFF, WINDOWS_DEVELOPMENT; export_handoff; native MSVC builder | Handoff archive hashes + actual Git history restore test | Source/docs/history portable; actual Windows MSVC restore/build pending |

## Изменения относительно v1

User DPAPI заменён по прямому решению пользователя на service vault. Audit-only stop отменён текущим поручением начинать код. G1 UI отложен до G5, чтобы не расходовать работу на оболочку до доказанного TUN. Separate profile/policy и JCS заменены **предложением**, не утверждением о сервере: единый signed-byte bundle. Старые server endpoints не менялись. Все OPEN решения доступны в OPEN_DECISIONS.md.
