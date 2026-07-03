<script lang="ts">
    import { cubicOut } from "svelte/easing";
    import { Tween } from "svelte/motion";

    let { value, label }: { value: number; label: string } = $props();

    // каноничные цвета статов (pokemondb) — ключ = имя стата из PokéAPI
    const COLORS: Record<string, string> = {
        hp: "#ff5959",
        attack: "#f5ac78",
        defense: "#fae078",
        "special-attack": "#9db7f5",
        "special-defense": "#a7db8d",
        speed: "#fa92b2",
    };
    const NAMES: Record<string, string> = {
        hp: "HP",
        attack: "Attack",
        defense: "Defense",
        "special-attack": "Sp. Atk",
        "special-defense": "Sp. Def",
        speed: "Speed",
    };

    // $derived, а не const — иначе цвет/имя застынут на первом label
    // и «поедут» при переиспользовании инстанса (each без ключа / HMR)
    let color = $derived(COLORS[label] ?? "#888");
    let name = $derived(NAMES[label] ?? label);

    // твиним само значение: ширина и цифра читаются из него
    const t = new Tween(0, { duration: 600, easing: cubicOut });
    $effect(() => {
        t.target = value;
    });
</script>

<li class="stat">
    <span class="name">{name}</span>
    <div class="track">
        <div
            class="fill"
            style:width="{Math.min((t.current / 255) * 100, 100)}%"
            style:background-color={color}
        ></div>
    </div>
    <span class="num">{Math.round(t.current)}</span>
</li>

<style>
    .stat {
        display: grid;
        grid-template-columns: 66px 1fr 36px;
        align-items: center;
        gap: 0.7em;
        padding: 0.42em 0;
    }

    .name {
        font-size: 0.92em;
        opacity: 0.65;
    }

    .track {
        height: 9px;
        border-radius: 999px;
        background: rgba(0, 0, 0, 0.09);
        overflow: hidden;
    }

    .fill {
        height: 100%;
        border-radius: 999px;
    }

    .num {
        font-variant-numeric: tabular-nums;
        font-size: 1em;
        font-weight: 600;
        text-align: right;
    }

    @media (prefers-color-scheme: dark) {
        .track {
            background: rgba(255, 255, 255, 0.12);
        }
    }
</style>
