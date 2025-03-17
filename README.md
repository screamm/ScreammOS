# ScreammOS

Ett retro-modernt operativsystem med DOS-känsla, byggt från grunden i Rust.

## Projektbeskrivning

ScreammOS är ett experiment i att bygga ett operativsystem som kombinerar nostalgisk DOS-estetik med moderna funktioner. Projektet fokuserar på att skapa en retro-inspirerad upplevelse med fönsterhantering, olika visuella teman (inklusive CRT-effekter) och grundläggande operativsystemsfunktioner.

## Funktioner

- Textbaserad VGA-buffert med 16-färgers palett
- DOS-inspirerad gränssnittsdesign
- Fönsterhantering med överlappande fönster
- Olika visuella teman (DOS-klassiskt, Amber-terminal, Grön CRT)
- Möjlighet att aktivera CRT-effekter för maximal retrokänsla

## Teknisk information

- Utvecklat i Rust utan standardbibliotek (no_std)
- Körs direkt på hårdvaran (bare metal)
- Målarkitektur: x86_64 (med framtida planer för ARM)
- Använder bootloader-crate för booting
- VGA-textläge för 80x25 textbaserat gränssnitt

## Byggprocess

Instruktioner för hur man bygger och kör ScreammOS kommer att läggas till senare.

## Roadmap

- [ ] Grundläggande tangentbordshantering
- [ ] Kommandotolk (shell)
- [ ] Enkel filhantering
- [ ] Minneshantering och processer
- [ ] Fler applikationer i DOS-stil

## Licens

Information om licens kommer att läggas till senare.

---

*ScreammOS - Den Retro-moderna Upplevelsen* 