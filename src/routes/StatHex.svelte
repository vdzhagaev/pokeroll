<script lang="ts">
    import { Tween } from "svelte/motion";
    import { cubicOut } from "svelte/easing";

    let {
        stats,
        color,
    }: { stats: { name: string; baseStat: number }[]; color: string } = $props();

    const CX = 100;
    const CY = 100;
    const MAXR = 80; // радиус внешнего кольца (крупнее — меньше пустого поля)
    const MAX = 200; // визуальный «потолок» стата (базовые редко выше)

    const SHORT: Record<string, string> = {
        hp: "HP",
        attack: "Atk",
        defense: "Def",
        "special-attack": "SpA",
        "special-defense": "SpD",
        speed: "Spe",
    };

    // рост при появлении: все точки тянутся из центра наружу
    const grow = new Tween(0, { duration: 600, easing: cubicOut });
    $effect(() => {
        grow.target = 1;
    });

    // точка i-й оси на радиусе r; старт сверху (-90°), по 60° на вершину
    function point(i: number, r: number) {
        const a = ((-90 + i * 60) * Math.PI) / 180;
        return { x: CX + r * Math.cos(a), y: CY + r * Math.sin(a) };
    }
    function poly(radii: number[]) {
        return radii.map((r, i) => `${point(i, r).x},${point(i, r).y}`).join(" ");
    }

    // многоугольник значений (с учётом анимации grow)
    let valuePoints = $derived(
        poly(stats.map((s) => Math.min(s.baseStat / MAX, 1) * MAXR * grow.current)),
    );
    // подписи чуть снаружи вершин
    let labels = $derived(
        stats.map((s, i) => ({
            ...point(i, MAXR + 13),
            short: SHORT[s.name] ?? s.name,
            val: s.baseStat,
        })),
    );
</script>

<svg class="hex" viewBox="0 0 200 200">
    {#each [0.33, 0.66, 1] as f}
        <polygon class="grid" points={poly(stats.map(() => MAXR * f))} />
    {/each}

    {#each stats as _, i}
        {@const p = point(i, MAXR)}
        <line class="axis" x1={CX} y1={CY} x2={p.x} y2={p.y} />
    {/each}

    <polygon
        class="value"
        points={valuePoints}
        style:fill={color}
        style:stroke={color}
    />

    {#each labels as l}
        <text class="lbl" x={l.x} y={l.y - 4}>{l.short}</text>
        <text class="val" x={l.x} y={l.y + 6}>{l.val}</text>
    {/each}
</svg>

<style>
    .hex {
        width: 100%;
        height: auto;
        overflow: visible; /* подписи выходят за viewBox */
    }

    .grid {
        fill: none;
        stroke: rgba(128, 128, 128, 0.25);
        stroke-width: 1;
    }

    .axis {
        stroke: rgba(128, 128, 128, 0.2);
        stroke-width: 1;
    }

    .value {
        fill-opacity: 0.35;
        stroke-width: 2;
        stroke-linejoin: round;
    }

    .lbl,
    .val {
        text-anchor: middle;
        fill: currentColor;
        font-variant-numeric: tabular-nums;
    }

    .lbl {
        font-size: 8px;
        font-weight: 700;
        opacity: 0.6;
    }

    .val {
        font-size: 9px;
        font-weight: 700;
        opacity: 0.9;
    }
</style>
