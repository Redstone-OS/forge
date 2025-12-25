//! Testes da Biblioteca de Base do Kernel (klib)
//!
//! # Por que testar?
//! O klib contém ferramentas usadas por todos os outros módulos. Um erro no parser de inteiros ou na
//! manipulação de bits pode causar erros silenciosos e indetectáveis em drivers ou no gerenciador de memória.
//!
//! # Lista de Testes Futuros:
//!
//! 1. `test_string_manipulation`:
//!    - O que: Testar split, trim e conversão de strings num ambiente sem `std`.
//!    - Por que: O VFS e o parser do Kernel Command Line dependem fortemente disso para funcionar.
//!
//! 2. `test_bit_ops_safety`:
//!    - O que: Verificar funções de set/clear bit em limites de palavra (32/64 bits).
//!    - Por que: O Bitmap do PMM e as Page Tables do VMM usam essas funções para gerenciar permissões.
//!
//! 3. `test_alignment_helpers`:
//!    - O que: Validar funções que alinham endereços para cima/baixo (ex: `align_up(0x123, 0x1000)`).
//!    - Por que: Erros de alinhamento causam Page Faults imediatos em muitas estruturas de hardware x86_64.
//!
//! 4. `test_checksum_integrity`:
//!    - O que: Calcular e verificar somas de verificação simples em buffers.
//!    - Por que: Útil para validar a integridade de tabelas ACPI ou headers de boot antes de processá-los.
