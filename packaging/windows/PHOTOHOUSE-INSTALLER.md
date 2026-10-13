# Instalador PhotoHouse para Windows

O artefato para compartilhar é `dist/PhotoHouse-Setup-0.7.0-x64.exe` na pasta
externa do projeto. É um instalador offline: contém o MSI, o editor, o CLI e o GIF.
Requer Windows 10/11 de 64 bits. Não requer Rust, WiX ou .NET no computador de destino.

Ao abrir, mostra o GIF original, a frase “Quase lá, estou instalando o seu app!”
em Segoe UI (com fallback embutido) e uma barra animada verde `#BFFE00`. A barra é
indeterminada durante a transação do Windows Installer; não inventa porcentagens.
Só mostra conclusão após sucesso do MSI. Código 3010 pede reinicialização sem
reiniciar automaticamente. O app só abre quando a pessoa clica em “Abrir PhotoHouse”.

Instala por usuário em `%LOCALAPPDATA%\Programs\PhotoHouse`, cria atalhos no menu
Iniciar e na área de trabalho e aparece em Configurações > Aplicativos para
desinstalação. O Windows Installer gerencia rollback, reparo e remoção dos arquivos.
Não altera as associações de imagens nem exclui documentos/preferências pessoais.
Durante a instalação, o fechamento da janela fica bloqueado para evitar abandono
da transação; o processo não encerra o editor aberto à força.

Erros ficam visíveis na tela. Registro técnico:
`%LOCALAPPDATA%\PhotoHouse\Installer\install.log`. O arquivo temporário MSI é
removido após a operação; o Windows conserva seu próprio cache para desinstalar.

## Recompilar

1. Disponibilizar `PhotoHouse.exe` e `photocraft-cli.exe` compilados na pasta externa.
2. Extrair a distribuição oficial [WiX 3.14.1](https://github.com/wixtoolset/wix3/releases/tag/wix3141rtm)
   em `installer-tools/wix` nessa pasta. WiX é ferramenta de build, não é distribuído.
3. Usar Rust com toolchain Windows e compilador de recursos disponível.
4. Executar `./packaging/windows/build-photohouse.ps1` a partir do repositório interno.

O frontend fica em um workspace Cargo independente para não alterar o editor nem
o build WASM. O build usa Cargo.lock e cache offline; numa máquina nova, executar
`cargo fetch --locked --manifest-path packaging/windows/photohouse-setup/Cargo.toml`
primeiro. Para mudar a versão, manter alinhados o manifesto do frontend, o script,
a versão dos executáveis empacotados e o rodapé da interface.

## Verificação

Com `PHOTOHOUSE_MSI` apontando para o MSI e `CARGO_TARGET_DIR` para
`target/photohouse-setup`, executar `cargo test --offline --locked --manifest-path
packaging/windows/photohouse-setup/Cargo.toml`. Os testes cobrem códigos de retorno,
payload MSI e animação/loop do GIF.

`PhotoHouse-Setup-0.7.0-x64.exe --preview --screenshot C:\caminho\preview.png`
mostra e captura a interface real sem instalar. `--extract-msi C:\caminho\app.msi`
extrai o pacote para auditoria. Não executar o instalador sem `--preview` só para
ver a tela: a instalação começa automaticamente.

O instalador não tem assinatura digital. Para distribuição pública com identidade
verificada, assinar os executáveis e MSI com um certificado Authenticode da equipe
e, após embutir o MSI assinado, assinar também o EXE final. Sem certificado, o
Windows pode exibir aviso de editor desconhecido/SmartScreen. O arquivo está pronto
para compartilhar; hospedagem e assinatura não são provisionadas por este build.

## Resultado da validação em 2026-10-13

- Compilação Rust release e `cargo check`: aprovados.
- Três testes Rust: aprovados (GIF animado e loop, assinatura do payload MSI,
  códigos de sucesso/erro/reinicialização do Windows Installer).
- WiX ICE: sem erros. ICE61 informa a atualização deliberada da mesma versão;
  ICE91 informa os diretórios deliberadamente específicos do usuário.
- Instalação real em pasta de teste, hashes dos dois executáveis, destino dos dois
  atalhos, execução do CLI, registro MSI/cache de desinstalação, reparo e remoção:
  aprovados. Arquivo criado pelo usuário foi preservado. A instalação de teste
  foi removida. Evidência: `dist/installation-test.json` e logs nele indicados.
- O MSI extraído do EXE final tem SHA-256 idêntico ao pacote validado.
- A interface real foi capturada em `dist/installer-preview.png` e revisada.
- DLLs importadas são componentes do Windows; não foram encontrados runtimes
  de terceiros externos na tabela de importação dos três executáveis.

Teste feito neste Windows; não substitui uma matriz de máquinas, GPUs e políticas
corporativas. O editor empacotado é o executável 0.7.0 local fornecido, sem mudanças
no motor de edição. O arquivo `SHA256SUMS.txt` acompanha os dois artefatos.

O registro do pacote e sua presença em Aplicativos seguem o mecanismo de
[registro de aplicativos MSI da Microsoft](https://learn.microsoft.com/en-us/windows/win32/msi/configuring-add-remove-programs-with-windows-installer).
