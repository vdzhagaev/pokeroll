<script lang="ts">
    // Рисует спрайт сеткой ячеек, снятой с канваса.
    //  mode="ascii"  — в ячейке символ рампы по яркости, цвет = цвет пикселя
    //  mode="blocks" — квадратный пиксель: один цвет на ячейку (8/16-битный вид)
    // Размер ячейки — целое число px (floor(box/cols)); поэтому клетки стыкуются
    // без дробных щелей, и на скрытых статах (коробка больше) шаг просто крупнее.
    let {
        src,
        mode = "ascii",
        cols = 42,
    }: { src: string; mode?: "ascii" | "blocks"; cols?: number } = $props();

    type Cell = { ch: string; color: string; grad: string };
    let rows = $state<Cell[][]>([]);
    let failed = $state(false); // canvas «испачкан» (CORS) или картинка не грузится

    // измеряем коробку .art через ResizeObserver — ловит и ресайз от смены
    // класса у предка (скрытие статов), чего bind:clientWidth не гарантирует
    let wrapEl = $state<HTMLDivElement | null>(null);
    let boxW = $state(0);
    $effect(() => {
        const el = wrapEl;
        if (!el) return;
        const ro = new ResizeObserver((entries) => {
            const r = entries[0].contentRect;
            boxW = Math.floor(Math.min(r.width, r.height));
        });
        ro.observe(el);
        return () => ro.disconnect();
    });

    // целочисленный размер ячейки — ключ к отсутствию щелей
    let cell = $derived(boxW > 0 ? Math.max(1, Math.floor(boxW / cols)) : 0);

    // текст для копирования: символы по строкам, хвостовые пробелы срезаем
    let asciiText = $derived(
        rows.map((r) => r.map((c) => c.ch).join("").replace(/\s+$/, "")).join("\n"),
    );
    let copied = $state(false);
    async function copy() {
        try {
            await navigator.clipboard.writeText(asciiText);
            copied = true;
            setTimeout(() => (copied = false), 1200);
        } catch (e) {
            console.error(e);
        }
    }

    const RAMP = " .:-=+*#%@"; // по возрастанию плотности

    $effect(() => {
        const url = src;
        const m = mode;
        const w = cols;
        rows = [];
        failed = false;
        if (!url) return;

        const img = new Image();
        img.crossOrigin = "anonymous"; // нужно, чтобы читать пиксели без taint
        img.onload = () => {
            try {
                const h = w; // квадратная выборка: столько же строк, сколько столбцов
                // супер-сэмпл: рисуем в SS раз крупнее и усредняем SS×SS субпикселей
                // в один цвет ячейки => честная area-average, чистый цвет при низком
                // разрешении блоков (детальнее без роста числа пикселей)
                const SS = 4;
                const sw = w * SS;
                const sh = h * SS;
                const canvas = document.createElement("canvas");
                canvas.width = sw;
                canvas.height = sh;
                const ctx = canvas.getContext("2d");
                if (!ctx) return;
                ctx.imageSmoothingEnabled = true;
                ctx.imageSmoothingQuality = "high";
                ctx.drawImage(img, 0, 0, sw, sh);
                const data = ctx.getImageData(0, 0, sw, sh).data;

                // средний цвет блока (cx,cy); null = прозрачный.
                // цвет взвешиваем по alpha (premultiplied) — прозрачная кайма
                // не тянет края в тёмный
                const cellColor = (cx: number, cy: number) => {
                    let r = 0, g = 0, b = 0, aSum = 0, n = 0;
                    for (let dy = 0; dy < SS; dy++) {
                        for (let dx = 0; dx < SS; dx++) {
                            const i = ((cy * SS + dy) * sw + (cx * SS + dx)) * 4;
                            const a = data[i + 3];
                            r += data[i] * a;
                            g += data[i + 1] * a;
                            b += data[i + 2] * a;
                            aSum += a;
                            n++;
                        }
                    }
                    if (aSum / n < 40) return null; // блок в основном прозрачный
                    return {
                        r: Math.round(r / aSum),
                        g: Math.round(g / aSum),
                        b: Math.round(b / aSum),
                    };
                };

                const grid: Cell[][] = [];
                for (let y = 0; y < h; y++) {
                    const row: Cell[] = [];
                    for (let x = 0; x < w; x++) {
                        const c = cellColor(x, y);
                        if (!c) {
                            row.push({ ch: " ", color: "transparent", grad: "transparent" });
                            continue;
                        }
                        const rgb = `rgb(${c.r},${c.g},${c.b})`;
                        if (m === "blocks") {
                            row.push({ ch: "", color: "", grad: rgb });
                        } else {
                            const lum = (0.299 * c.r + 0.587 * c.g + 0.114 * c.b) / 255;
                            const ch =
                                RAMP[Math.min(RAMP.length - 1, Math.floor(lum * RAMP.length))];
                            row.push({ ch, color: rgb, grad: "" });
                        }
                    }
                    grid.push(row);
                }
                rows = grid;
            } catch {
                failed = true; // getImageData кинул — canvas tainted
            }
        };
        img.onerror = () => (failed = true);
        img.src = url;
    });
</script>

{#if failed}
    <img class="fallback" {src} alt="" />
{:else}
    <div class="wrap" class:term={mode === "ascii"} bind:this={wrapEl}>
        <div
            class="grid"
            class:ascii={mode === "ascii"}
            style:grid-template-columns={`repeat(${cols}, ${cell}px)`}
            style:grid-template-rows={`repeat(${rows.length}, ${cell}px)`}
            style:font-size={`${cell}px`}
            style:line-height={`${cell}px`}
            aria-hidden="true"
        >
            {#each rows as row}
                {#each row as c}
                    {#if mode === "blocks"}
                        <div class="px" style:background={c.grad}></div>
                    {:else}
                        <span class="ch" style:color={c.color}>{c.ch}</span>
                    {/if}
                {/each}
            {/each}
        </div>

        {#if mode === "ascii"}
            <button class="copy" onclick={copy} title="Copy ASCII">
                {copied ? "✓ copied" : "copy"}
            </button>
        {/if}
    </div>
{/if}

<style>
    .wrap {
        position: relative;
        width: 100%;
        height: 100%;
        min-width: 0; /* иначе min-content сетки не даёт .wrap сжаться -> */
        min-height: 0; /* -> ResizeObserver мерит старую ширину, картинка не уменьшается */
        overflow: hidden; /* клип переходного кадра, пока сетка пересчитывается */
        display: grid;
        place-items: center; /* фикс-грид центрируем в коробке */
    }

    /* ascii-режим — «терминал»: тёмная подложка, чтобы цветные глифы читались */
    .wrap.term {
        background: #14141a;
        border-radius: 14px;
    }

    .grid {
        display: grid;
        font-family: monospace;
    }
    .grid.ascii {
        font-weight: 700; /* жирнее => заметнее */
    }
    .px {
        width: 100%;
        height: 100%;
    }
    .ch {
        display: grid;
        place-items: center;
        overflow: hidden;
    }

    .copy {
        position: absolute;
        top: 6px;
        right: 6px;
        z-index: 2;
        padding: 0.32em 0.6em;
        font: 600 11px/1 Inter, sans-serif;
        color: #fff;
        border: 1px solid rgba(255, 255, 255, 0.28);
        border-radius: 7px;
        background: rgba(255, 255, 255, 0.12);
        cursor: pointer;
        transition: background 0.15s;
    }
    .copy:hover {
        background: rgba(255, 255, 255, 0.24);
    }

    .fallback {
        width: 100%;
        height: 100%;
        object-fit: contain;
        image-rendering: pixelated;
    }
</style>
