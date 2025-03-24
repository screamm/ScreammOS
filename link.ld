/* Grundläggande länkningsskript för ScreammOS */

ENTRY(_start)

SECTIONS {
    /* Börja vid adressen 1M */
    . = 1M;

    /* Koda aligneringen tydligt för varje sektion */
    .text ALIGN(4K) : {
        *(.text.boot)  /* Placera boot-koden först */
        *(.text .text.*)
        . = ALIGN(4K);
    }

    /* Sedan .rodata-sektionen med konstanter */
    .rodata ALIGN(4K) : {
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }

    /* .data-sektionen med initialiserade variabler */
    .data ALIGN(4K) : {
        *(.data .data.*)
        . = ALIGN(4K);
    }

    /* .bss-sektionen med oinitialiserade variabler */
    .bss ALIGN(4K) : {
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(4K);
    }

    /* Definiera slutet på kerneln */
    _end = .;

    /* Kasta bort dessa sektioner för att förbättra kompatibilitet */
    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
        *(.note.*)
    }
} 