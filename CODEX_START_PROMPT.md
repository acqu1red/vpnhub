# Продолжение ScrumeVPN

Прочитать AGENTS.md, CODEX_MASTER_SPEC.md, docs/DECISIONS.md, docs/STATUS.md и docs/EXECUTION_PLAN.md. Исторический audit-only prompt сохранён в docs/archive/START_PROMPT_v1.md и заменён прямым поручением пользователя проводить аудит **и начинать реализацию**.

Продолжать ближайшую незаблокированную задачу до проверенного результата. Не запрашивать повторное одобрение уже разрешённой локальной реализации. Отдельно учитывать SOURCE/PROPOSAL/OPEN/VERIFIED; не выдавать core tests/cross-check за Windows SCM/TUN/DPAPI/MSI E2E. Production не менять без явного допуска.

Пользователь утвердил pre-login boot-connect после initial enrollment, service-only token vault (machine DPAPI+ACL), server policy для отключения auto_connect. VMware Fusion разрешён для tests; ARM guest не закрывает выпускную x64 матрицу.

Выполнить tools/dev.py doctor/test, обновить STATUS/TRACEABILITY и минимальный следующий пакет внешних зависимостей. Не создавать фиктивный готовый MSI и не копировать GPL VPN code или RustVPN с неподтверждённой лицензией.
