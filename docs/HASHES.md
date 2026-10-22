# Hashes / Historial reescrito

**ES:** El 2026-10-22 se reescribió todo el historial de este repositorio (git-filter-repo) para quitar datos del host y del usuario (nombre de máquina, usuario, uid, rutas de cgroup, versión de kernel, y un par TLS de prueba con clave privada). Los commits cambiaron de hash; los documentos de auditoría citan los hashes viejos. Esta tabla da la equivalencia.

**EN:** On 2026-10-22 this repository's full history was rewritten (git-filter-repo) to remove host and user data. Commit hashes changed; audit documents cite the old hashes. This table maps old → new.

| Viejo / Old | Nuevo / New | Fecha (UTC) | Mensaje / Message |
|---|---|---|---|
| `e57e321` | `e57e321` | 2026-10-21T01:44:25Z | 06: detector KS de canal de temporización, generadores sintéticos y evaluación FPR/TPR |
| `7b5c59e` | `1a1ca03` | 2026-10-21T02:04:41Z | 06: uprobe real SSL_write + captura TLS local con FPR/TPR medidos |
| `2483c5c` | `abbfc2b` | 2026-10-21T02:46:50Z | 06: captura-real.sh sale 0 (veredicto != fallo), runs/ gitignorado, README con rango de p en vez de valor único |
| `d9fe2cb` | `d87979c` | 2026-10-21T03:00:39Z | 06: experimento de valor — adversario imitador (mimic) y tabla TPR@FPR |
| `a8d8a8e` | `d39cc5f` | 2026-10-21T03:02:55Z | 06: robustez del valor con 5 semillas (rango min–max y media) |
| `fdca11a` | `842a4b0` | 2026-10-22 | Licencia Apache-2.0 (mensaje reescrito con formato nuevo / rewritten message) |
