# Empacotamento do PhotoHouse

Os pacotes que o **House Studio** instala: um `.dmg` para macOS e um `.zip` portátil para
Windows. O Studio baixa o arquivo pelo ticket do House Room, confere o SHA-256 e instala
(código no repositório HOUSE-STUDIO: `crates/native/src/detect.rs`, `install.rs`,
`portable.rs` e `room.rs`).

| Sistema | Script | Arquivo gerado em `dist/release/` |
|---|---|---|
| macOS (Apple Silicon + Intel) | `packaging/macos/package.sh` | `photohouse-<versão>-macos-universal.dmg` |
| Windows x64 | `packaging/windows/package.ps1` | `photohouse-<versão>-windows-x64-portable.zip` |

O macOS gera também `photohouse-cli-<versão>-macos-universal.zip` (o CLI sem interface). Ele não
é usado pelo Studio.

## O que o House Studio exige

**macOS (`.dmg`)**

- `PhotoHouse.app` na raiz do volume (o Studio monta a imagem e copia o app para Aplicativos).
- `Contents/Info.plist` com `CFBundleIdentifier` = `br.com.house.photohouse`,
  `PhotoHouseVersion` = versão completa (por exemplo `0.8.2` ou `0.9.0-rc.1`) e
  `CFBundleShortVersionString` = só os números (`0.9.0`).
- Executável `Contents/MacOS/PhotoHouse`.

**Windows (`.zip`)**

- `photohouse.exe` na **raiz** do arquivo, sem pasta por cima. Junto vão `photohouse-cli.exe`,
  README e licenças. Os binários usam o runtime C estático: não há DLL extra.
- O nome termina em `-portable.zip`. É por esse sufixo que o Studio reconhece um zip portátil;
  um nome terminado só em `.zip` é ignorado.
- Sem `portable.txt`: o Studio é dono da pasta de instalação e troca os arquivos a cada
  atualização, então as preferências ficam na pasta normal do usuário (`%APPDATA%`).

O nome do arquivo segue `photohouse-<versão>-<sistema>-<arquitetura>`. Os crates e binários
internos continuam `photocraft`/`photocraft-cli` (herança do PhotoCraft); os scripts renomeiam
os arquivos entregues.

## Versão

Vem de `[workspace.package] version` no `Cargo.toml` da raiz. No CI, defina
`PHOTOHOUSE_VERSION` para usar outra (por exemplo a tag da Release; um `v` no início é
removido). `PHOTOCRAFT_VERSION`, o nome antigo, ainda funciona.

## macOS

Pré-requisitos:

- Um Mac (Apple Silicon ou Intel) com macOS 11 ou mais novo.
- Xcode Command Line Tools: `xcode-select --install` (fornece `lipo`, `codesign`, `hdiutil`,
  `plutil`).
- Rust estável (o mínimo está em `rust-version` no `Cargo.toml`) com os dois alvos:

  ```sh
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  ```

Gerar o pacote universal (compila os dois alvos e junta com `lipo`):

```sh
packaging/macos/package.sh
```

Opções: `--arch aarch64` ou `--arch x86_64` geram um pacote de uma arquitetura só
(`-macos-aarch64.dmg`, `-macos-x86_64.dmg`); `--skip-build` reaproveita binários já
compilados em `target/<alvo>/release/`.

Conferir o resultado como ele será baixado (monta o DMG somente leitura, confere
`PhotoHouse.app`, o bundle id, as chaves de versão e a assinatura):

```sh
packaging/macos/verify.sh
```

**Assinatura.** A House ainda não tem certificado Apple. Sem `MACOS_SIGN_IDENTITY`, o script
assina ad-hoc (`codesign --force --deep --sign -`), sem notarização, e funciona sem
`APPLE_CERTIFICATE` definido. Num Mac que baixou o DMG pelo navegador, o Gatekeeper bloqueia a
primeira abertura (botão direito > Abrir, ou Ajustes do Sistema > Privacidade e Segurança).
Quando houver certificado: `packaging/macos/import-cert.sh` importa `APPLE_CERTIFICATE` num
keychain temporário (`photohouse-signing.keychain-db`) e `APPLE_ID`, `APPLE_PASSWORD` e
`APPLE_TEAM_ID` ativam a notarização.

A janela do DMG (fundo e posição dos ícones) vem de `packaging/macos/dmg/`; veja o README de lá.

## Windows

Pré-requisitos (numa máquina Windows 10/11 x64):

- Rust estável com o toolchain MSVC e o alvo: `rustup target add x86_64-pc-windows-msvc`.
- Visual Studio Build Tools com a carga "Desenvolvimento para desktop com C++" e o Windows SDK
  (o `rc.exe` grava ícone e VERSIONINFO no `.exe`).
- PowerShell 5.1 (já vem no Windows) ou PowerShell 7 (`pwsh`).
- Git, para gravar o commit no `--version`.

Gerar o zip:

```powershell
powershell -ExecutionPolicy Bypass -File packaging\windows\package.ps1
```

Opções: `-Arch x86` ou `-Arch arm64` (o Studio procura `windows-x64` primeiro), `-SkipBuild`
reaproveita `target\<alvo>\release\`. O script confere o cabeçalho PE dos binários, monta o zip
plano, confere que `photohouse.exe` está na raiz, roda `photohouse-cli.exe --version` e imprime
o SHA-256.

**Assinatura.** Não há certificado de assinatura de código. Sem `WINDOWS_CERTIFICATE` ou as
variáveis `AZURE_*`, `sign.ps1` avisa e deixa os arquivos sem assinatura; o Windows pode mostrar
o aviso do SmartScreen.

O instalador personalizado com a mascote (`PhotoHouse-Setup-<versão>-x64.exe`) é outro
processo, descrito em `packaging/windows/PHOTOHOUSE-INSTALLER.md`.
`photocraft.wxs`, `photocraft.en-us.wxl` e `check-icons.ps1` são a receita MSI herdada do
PhotoCraft: o `package.ps1` não as usa mais, mas os testes do editor (`apps/photocraft/src/app_icon.rs`)
leem o `.wxs`, por isso continuam no repositório.

## Outros alvos

- Linux: `packaging/linux/package.sh` (AppImage, deb, rpm, tar.gz) e
  `packaging/linux/flatpak-bundle.sh`. O id de app Linux continua `ai.storyteller.photocraft`:
  ele precisa ser igual ao `APP_ID` da janela (`apps/photocraft/src/main.rs`), que também dá
  nome à pasta onde o editor guarda o estado das janelas. Trocar exige uma migração no editor.
- FreeBSD: `packaging/freebsd/package.sh`.
- Web: `packaging/web/package.sh` gera `photohouse-web-<versão>.zip`; o `Dockerfile` da raiz
  serve a mesma versão web com nginx (`packaging/web/README.md`).
