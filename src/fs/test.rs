//! Testes do Sistema de Arquivos (VFS/RFS)
//!
//! # Por que testar?
//! O VFS é a interface universal para arquivos, dispositivos e pipes. Corrupção aqui significa perda de dados
//! ou carregamento de drivers errados. Testar mount points garante que o kernel saiba onde cada disco "mora".
//!
//! # Lista de Testes Futuros:
//!
//! 1. `test_vfs_lookup_path`:
//!    - O que: Buscar caminhos válidos, inválidos e com múltiplos separadores (ex: `//system/./core/init`).
//!    - Por que: Garante que o parser de caminho é robusto e resolve corretamente a hierarquia de diretórios.
//!
//! 2. `test_mount_isolation`:
//!    - O que: Montar dois sistemas de arquivos em pontos diferentes e garantir que um não "vaze" para o outro.
//!    - Por que: Fundamental para a segurança e organização do sistema (ex: `/tmp` deve ser isolado do `/root`).
//!
//! 3. `test_handle_management`:
//!    - O que: Abrir e fechar arquivos repetidamente até atingir o limite de handles.
//!    - Por que: Previne vazamentos de memória e garante que os recursos do kernel são liberados após o uso.
//!
//! 4. `test_read_beyond_eof`:
//!    - O que: Tentar ler bytes além do tamanho informado pelo arquivo.
//!    - Por que: Verifica se os drivers de FS respeitam os limites físicos do arquivo, evitando leitura de lixo do disco.
