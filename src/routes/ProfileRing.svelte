<script lang="ts">
    import { untrack } from "svelte";
    import { scale } from "svelte/transition";
    import { backOut } from "svelte/easing";

    let { level, progress }: { level: number; progress: number } = $props();

    const R = 20;
    const C = 2 * Math.PI * R; // длина окружности

    // то, что реально показываем — со своей анимацией level-up
    let displayLevel = $state(level);
    let displayProgress = $state(progress);
    let celebrating = $state(false);
    let snapping = $state(false); // мгновенный сброс дуги без перехода

    let offset = $derived(C * (1 - Math.min(displayProgress, 100) / 100));

    // цель, к которой догоняемся; воркер крутит, пока не сойдётся.
    // начальный захват осознанный — эффект перезапишет цель до первого drain()
    let targetLevel = untrack(() => level);
    let targetProgress = untrack(() => progress);
    let running = false;

    const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

    async function drain() {
        if (running) return; // воркер уже крутится — он подхватит новую цель
        running = true;
        try {
            // догоняем по уровням, по одному level-up за проход
            while (displayLevel < targetLevel) {
                displayProgress = 100; // доливаем текущий уровень до конца
                await wait(520);

                celebrating = true; // галочка + заливка центра
                await wait(700);

                // сброс на следующий уровень мгновенно, без отката дуги
                snapping = true;
                celebrating = false;
                displayLevel += 1;
                displayProgress = 0;
                await wait(30);
                snapping = false;
                await wait(20); // маленький вдох перед следующим доливом
            }
            // на нужном уровне — доливаем до остатка (цель могла подрасти по пути)
            displayProgress = targetProgress;
        } finally {
            running = false;
        }
        if (displayLevel < targetLevel) drain(); // цель ушла вперёд — догоняем
    }

    $effect(() => {
        targetLevel = level;
        targetProgress = progress;
        drain();
    });
</script>

<div class="ring" class:celebrating title="Level {displayLevel}">
    <svg viewBox="0 0 48 48" width="46" height="46">
        <circle class="track" cx="24" cy="24" r={R} />
        <circle
            class="prog"
            class:snap={snapping}
            cx="24"
            cy="24"
            r={R}
            stroke-dasharray={C}
            stroke-dashoffset={offset}
        />
    </svg>

    <div class="center">
        {#if celebrating}
            <svg
                class="check"
                viewBox="0 0 24 24"
                in:scale={{ duration: 300, easing: backOut }}
            >
                <path d="M5 12.5l4 4 10-10" />
            </svg>
        {:else}
            <span class="lvl">{displayLevel}</span>
        {/if}
    </div>
</div>

<style>
    .ring {
        position: relative;
        width: 46px;
        height: 46px;
    }

    /* поворот только кольца, не галочки внутри (та не прямой ребёнок .ring) */
    .ring > svg {
        transform: rotate(-90deg);
    }

    .track {
        fill: none;
        stroke: rgba(0, 0, 0, 0.12);
        stroke-width: 4;
    }

    .prog {
        fill: none;
        stroke: #8b5cf6;
        stroke-width: 4;
        stroke-linecap: round;
        transition: stroke-dashoffset 500ms ease-out;
    }

    .prog.snap {
        transition: none;
    }

    .center {
        position: absolute;
        inset: 5px;
        border-radius: 50%;
        display: grid;
        place-items: center;
        line-height: 1; /* иначе цифра оседает вниз по большому line-height */
        transition: background-color 250ms ease;
    }

    .ring.celebrating .center {
        background-color: #8b5cf6;
    }

    .lvl {
        font-size: 0.9em;
        font-weight: 800;
    }

    .check {
        width: 22px;
        height: 22px;
        fill: none;
        stroke: #fff;
        stroke-width: 2.5;
        stroke-linecap: round;
        stroke-linejoin: round;
    }

    @media (prefers-color-scheme: dark) {
        .track {
            stroke: rgba(255, 255, 255, 0.15);
        }
    }
</style>
