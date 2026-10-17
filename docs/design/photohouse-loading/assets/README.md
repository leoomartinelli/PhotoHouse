# Housinho — animação e alternativa estática

`housinho-base.webp` é uma cópia comprimida, com proporções e composição
preservadas, da imagem **ILLUSTRATOR-JANELA-DE-ABETURA.png** (912 × 1055) fornecida
pelo solicitante nesta conversa em 17/10/2026 para testar o design da issue #5.

O solicitante autorizou o uso no protótipo e seu versionamento nesta branch/PR.
Autor e licença pública não foram informados. Esta autorização específica não
declara a arte como domínio público nem concede uma licença geral a terceiros.

`housinho.gif` agora é gerado do vídeo **ANIMAÇÃO.mp4**, fornecido pelo mesmo
solicitante em 17/10/2026 para melhorar a qualidade da animação. Substitui o GIF
anterior de 490 × 566. Tem 720 × 832 pixels, 52 quadros de 80 ms (4,16 segundos),
paleta otimizada e reprodução única. A mesma autorização específica de uso no
protótipo e versionamento nesta branch/PR se aplica; nenhuma licença pública
ou autoria adicional foi informada. O MP4 original não é modificado.

`housinho-frames.webp` contém os 52 quadros completos do GIF em uma grade de
8 × 7 células de 720 × 832 pixels. É produzido por `../build_frames.py` diretamente
do MP4, em cores completas, para evitar a perda de paleta do GIF. O preview usa a grade para sincronizar
os quadros à contagem e parar no último quadro em 100%. A mesma origem e autorização
do vídeo se aplicam ao sprite. Movimento reduzido mostra o quadro final.
O solicitante autorizou posteriormente a integração local da tela de abertura
nativa em 17/10/2026. O sprite é compartilhado pelo estudo e pelo aplicativo
desktop (`apps/photocraft/src/startup_splash.rs`); não foi adicionado ao instalador.
