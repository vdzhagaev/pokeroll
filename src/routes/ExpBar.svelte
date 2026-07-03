<script lang="ts">
    import { cubicOut } from "svelte/easing";
    import { Tween } from "svelte/motion";

    let { value }: { value: number } = $props();

    // один прогресс 0→1, с задержкой: играет в финале, после статов.
    // из него читаем и ширину полосы, и набегающую цифру.
    const p = new Tween(0, { duration: 900, delay: 700, easing: cubicOut });
    $effect(() => {
        p.target = 1;
    });
</script>

<div class="exp-row">
    <span class="tag">EXP</span>
    <div class="track">
        <div class="fill" style:width="{p.current * 100}%"></div>
    </div>
    <span class="gain">+{Math.round(p.current * value)}</span>
</div>

<style>
    .exp-row {
        margin-top: 1.1em;
        display: grid;
        grid-template-columns: 34px 1fr 44px;
        align-items: center;
        gap: 0.55em;
        width: 100%;
    }

    .tag {
        font-size: 0.72em;
        font-weight: 700;
        letter-spacing: 0.06em;
        color: #3d9bfd;
    }

    .track {
        height: 7px;
        border-radius: 999px;
        background: rgba(0, 0, 0, 0.09);
        overflow: hidden;
    }

    .fill {
        height: 100%;
        border-radius: 999px;
        background: linear-gradient(90deg, #3d9bfd, #63c7ff);
    }

    .gain {
        font-variant-numeric: tabular-nums;
        font-size: 0.82em;
        font-weight: 700;
        text-align: right;
        color: #3d9bfd;
    }

    @media (prefers-color-scheme: dark) {
        .track {
            background: rgba(255, 255, 255, 0.12);
        }
    }
</style>
