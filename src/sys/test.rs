//! Testes de Informações Globais do Sistema (Sys)
//!
//! # Por que testar?
//! O módulo sys fornece metadados sobre o hardware e o kernel. Se a contagem de memória total ou a
//! versão do kernel estiverem erradas, programas de usuário (como o `top` ou `uname`) darão informações falsas.
//!
//! # Lista de Testes Futuros:
//!
//! 1. `test_uptime_consistency`:
//!    - O que: Verificar se o timer de uptime aumenta de forma linear e não retrocede.
//!    - Por que: Vital para logs de sistema e agendamento de eventos futuros.
//!
//! 2. `test_memory_stats_accuracy`:
//!    - O que: Comparar a soma (Memória Usada + Memória Livre) com o total reportado pelo Bootloader.
//!    - Por que: Detecta "vazamentos de memória físicos" onde o kernel perde o rastro de algumas regiões da RAM.
//!
//! 3. `test_cpu_info_parsing`:
//!    - O que: Verificar se o nome do processador e as extensões (SSE, AVX) foram detectadas corretamente.
//!    - Por que: Garante que o kernel saiba quais otimizações de hardware ele pode usar com segurança.
//!
//! 4. `test_version_string`:
//!    - O que: Validar o formato da string de versão do Redstone OS.
//!    - Por que: Importante para compatibilidade de pacotes e identificação de builds em relatórios de erro.
