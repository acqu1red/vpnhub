# ADR-0002: boot и vault службы

Статус: SOURCE D002–D005; механизм PROPOSAL.

Вариант A user DPAPI + login-only отклонён требованием boot-connect. Вариант B machine DPAPI + SYSTEM/service SID ACL выбран для первой реализации. Вариант C TPM/cert enrollment оставлен для managed fleet hardening. Initial OTP выполняется после логина; служба сохраняет token, owner SID и enrollment binding. На boot service использует vault без user profile и до запуска GUI. Signed auto_connect=false запрещает boot connect. Неавторизованный ПК не подключается.

Важно: CRYPTPROTECT_LOCAL_MACHINE позволяет любому пользователю компьютера расшифровать доступный ciphertext. Защиту service-only создаёт ACL на directory/file, trusted paths, запрет reparse substitution и эксклюзивный доступ службы. Service process обращается к DPAPI после RevertToSelf. Секреты в памяти zeroize где возможно; dump/local admin не входят в гарантии. Тесты: другой SID не читает файл; копия ciphertext не уходит в GUI; reboot без user logon; policy disables boot; reinstall/account switch; corrupted blob fail closed.
