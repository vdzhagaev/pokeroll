<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import { check, type Update } from "@tauri-apps/plugin-updater";
    import { relaunch } from "@tauri-apps/plugin-process";
    import { onMount } from "svelte";
    import StatBar from "./StatBar.svelte";
    import StatHex from "./StatHex.svelte";
    import ProfileRing from "./ProfileRing.svelte";
    import AsciiSprite from "./AsciiSprite.svelte";

    interface Stat {
        name: string;
        baseStat: number;
    }

    interface Pokemon {
        id: number;
        name: string;
        types: string[];
        stats: Stat[];
        sprite: string;
        baseExperience: number;
        weight: number;
        height: number;
    }

    // --- персист в localStorage (тип 1: настройки + прогресс).
    // покедекс (seen) пока тоже тут; позже переедет в SQLite (rung 4).
    const KEY = "pokeroll:";
    function load<T>(k: string, fallback: T): T {
        if (typeof localStorage === "undefined") return fallback;
        try {
            const raw = localStorage.getItem(KEY + k);
            return raw === null ? fallback : (JSON.parse(raw) as T);
        } catch {
            return fallback;
        }
    }
    function save(k: string, v: unknown) {
        if (typeof localStorage === "undefined") return;
        try {
            localStorage.setItem(KEY + k, JSON.stringify(v));
        } catch {}
    }

    let pokemon = $state<Pokemon | null>(null);
    let isNew = $state(false);
    let loading = $state(false); // идёт сетевой запрос
    let imgLoaded = $state(false); // спрайт догрузился
    let settingsOpen = $state(false);

    // виденные (покедекс) + опыт тренера — грузим из хранилища
    const seen = new Set<number>(load<number[]>("seen", []));
    let totalXp = $state(load("xp", 0));

    // всплывашки «+N EXP»: список активных, каждая со своим id и позицией —
    // несколько могут лететь одновременно, удаляются по концу анимации
    type ExpFloat = { id: number; value: number; x: number; y: number };
    let floats = $state<ExpFloat[]>([]);
    let floatSeq = 0;
    function spawnExp(value: number) {
        const x = 12 + Math.random() * 72; // % по горизонтали
        const y = 18 + Math.random() * 54; // % по вертикали
        floats.push({ id: floatSeq++, value, x, y });
    }
    function removeFloat(id: number) {
        floats = floats.filter((f) => f.id !== id);
    }

    const XP_PER_LEVEL = 100;
    let level = $derived(Math.floor(totalXp / XP_PER_LEVEL) + 1);
    let levelProgress = $derived(((totalXp % XP_PER_LEVEL) / XP_PER_LEVEL) * 100);

    // режим отображения статов
    type StatView = "bar" | "hex" | "none";
    const STAT_VIEWS: StatView[] = ["bar", "hex", "none"];
    const STAT_LABELS: Record<StatView, string> = {
        bar: "Bars",
        hex: "Hex",
        none: "Hide",
    };
    let statView = $state<StatView>(load("statView", "bar"));

    // режим картинки
    type ImageMode = "art" | "pixel" | "ascii";
    const IMAGE_MODES: ImageMode[] = ["art", "pixel", "ascii"];
    const IMAGE_LABELS: Record<ImageMode, string> = {
        art: "Art",
        pixel: "Pixel",
        ascii: "ASCII",
    };
    let imageMode = $state<ImageMode>(load("imageMode", "art"));

    // зерно пикселизации/ascii — число клеток в ряд (больше = детальнее)
    let pixelCols = $state(load("pixelCols", 24));
    let asciiCols = $state(load("asciiCols", 32));

    // поверх всех окон — переключаемо в настройках
    let alwaysOnTop = $state(load("alwaysOnTop", true));

    // пишем обратно при любом изменении
    $effect(() => save("statView", statView));
    $effect(() => save("imageMode", imageMode));
    $effect(() => save("xp", totalXp));
    $effect(() => save("pixelCols", pixelCols));
    $effect(() => save("asciiCols", asciiCols));
    // сохраняем и сразу применяем к окну (в т.ч. восстанавливаем при старте)
    $effect(() => {
        save("alwaysOnTop", alwaysOnTop);
        getCurrentWindow().setAlwaysOnTop(alwaysOnTop);
    });

    // тащим окно за карту; по кнопкам/инпутам — не таскаем
    function startDrag(e: MouseEvent) {
        if (e.button !== 0) return;
        if ((e.target as HTMLElement).closest("button, input, a")) return;
        getCurrentWindow().startDragging();
    }

    // --- обновления (updater-плагин) ---
    // фаза управляет модалкой; "idle" — модалки нет
    type UpdatePhase =
        | "idle"
        | "checking"
        | "available"
        | "installing"
        | "uptodate"
        | "error";
    let phase = $state<UpdatePhase>("idle");
    let updateInfo = $state<Update | null>(null);
    let updateError = $state("");
    let checking = $state(false);
    let updating = $state(false);
    let autoCheck = $state(load("autoCheck", true));
    let updateToken = 0; // инвалидирует текущую проверку при отмене
    $effect(() => save("autoCheck", autoCheck));

    async function checkUpdate(manual = false) {
        if (checking || updating) return;
        const token = ++updateToken;
        checking = true;
        if (manual) phase = "checking"; // авто-проверка молчит, если апдейта нет
        try {
            const u = await check();
            if (token !== updateToken) return; // отменили — результат игнорим
            if (u) {
                updateInfo = u;
                phase = "available";
            } else if (manual) {
                phase = "uptodate";
            }
        } catch (e) {
            console.error(e);
            if (token === updateToken && manual) {
                updateError = String(e);
                phase = "error";
            }
        } finally {
            if (token === updateToken) checking = false;
        }
    }

    function cancelUpdate() {
        updateToken++; // идущая проверка больше не применится
        checking = false;
        phase = "idle";
    }

    async function installUpdate() {
        if (!updateInfo || updating) return;
        updating = true;
        phase = "installing";
        try {
            await updateInfo.downloadAndInstall(); // качает, проверяет подпись, ставит
            await relaunch();
        } catch (e) {
            console.error(e);
            updateError = String(e);
            phase = "error";
            updating = false;
        }
    }

    onMount(() => {
        if (autoCheck) checkUpdate(false); // тихая авто-проверка при старте
    });

    // цвет по типу — рамка, бейджи, гекс, контролы
    const TYPE_COLORS: Record<string, string> = {
        normal: "#a8a878",
        fire: "#f08030",
        water: "#6890f0",
        electric: "#f8d030",
        grass: "#78c850",
        ice: "#98d8d8",
        fighting: "#c03028",
        poison: "#a040a0",
        ground: "#e0c068",
        flying: "#a890f0",
        psychic: "#f85888",
        bug: "#a8b820",
        rock: "#b8a038",
        ghost: "#705898",
        dragon: "#7038f8",
        dark: "#705848",
        steel: "#b8b8d0",
        fairy: "#ee99ac",
    };
    let typeColors = $derived(
        pokemon ? pokemon.types.map((t) => TYPE_COLORS[t] ?? "#888") : ["#888"],
    );
    let accent = $derived(typeColors[0]);
    let borderGradient = $derived(
        `linear-gradient(135deg, ${typeColors[0]}, ${typeColors[1] ?? typeColors[0]})`,
    );

    async function roll() {
        if (loading) return;
        loading = true;
        try {
            const p = await invoke<Pokemon>("roll");
            isNew = !seen.has(p.id);
            seen.add(p.id);
            if (isNew) save("seen", [...seen]); // покедекс на диск
            totalXp += p.baseExperience;
            spawnExp(p.baseExperience); // всплывашка в случайном месте
            imgLoaded = false;
            pokemon = p;
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    }
</script>

<main class="container">
    <!-- data-tauri-drag-region: тащить безрамочное окно за фон карты -->
    <div
        class="card"
        class:hide-stats={statView === "none"}
        style:--accent={accent}
        style:--border-gradient={borderGradient}
        onmousedown={startDrag}
        data-tauri-drag-region
    >
        <!-- уровень слева-сверху -->
        <div class="corner tl">
            <ProfileRing {level} progress={levelProgress} />
        </div>
        <!-- шестерёнка + «спрятать в трей» справа-сверху -->
        <div class="corner tr">
            <button
                class="icon-btn"
                onclick={() => (settingsOpen = !settingsOpen)}
                title="Settings"
                aria-label="Settings"
            >
                <svg viewBox="0 0 24 24">
                    <circle cx="12" cy="12" r="3" />
                    <path
                        d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z"
                    />
                </svg>
            </button>
            <button
                class="icon-btn"
                onclick={() => getCurrentWindow().hide()}
                title="Hide to tray"
                aria-label="Hide to tray"
            >
                <svg viewBox="0 0 24 24">
                    <path d="M18 6 6 18M6 6l12 12" />
                </svg>
            </button>
        </div>

        {#if pokemon}
            {#key pokemon.id}
                <div class="title">
                    <span class="dexno">#{String(pokemon.id).padStart(3, "0")}</span>
                    <h2 class="name">{pokemon.name}</h2>
                    {#if isNew}
                        <span class="new-badge">NEW</span>
                    {/if}
                </div>

                <div class="body">
                    <div class="left">
                        <div class="art">
                            {#if imageMode === "pixel"}
                                <AsciiSprite src={pokemon.sprite} mode="blocks" cols={pixelCols} />
                            {:else if imageMode === "ascii"}
                                <AsciiSprite src={pokemon.sprite} mode="ascii" cols={asciiCols} />
                            {:else}
                                {#if !imgLoaded}
                                    <div class="skeleton"></div>
                                {/if}
                                <img
                                    class="sprite"
                                    class:loaded={imgLoaded}
                                    src={pokemon.sprite}
                                    alt={pokemon.name}
                                    onload={() => (imgLoaded = true)}
                                />
                            {/if}
                        </div>

                        <div class="types">
                            {#each pokemon.types as t}
                                <span
                                    class="type"
                                    style:background-color={TYPE_COLORS[t] ?? "#888"}
                                >
                                    {t}
                                </span>
                            {/each}
                        </div>

                        <div class="meta">
                            <span><b>{(pokemon.weight / 10).toFixed(1)}</b> kg</span>
                            <span><b>{(pokemon.height / 10).toFixed(1)}</b> m</span>
                        </div>
                    </div>

                    {#if statView === "bar"}
                        <ul class="stats">
                            {#each pokemon.stats as s (s.name)}
                                <StatBar value={s.baseStat} label={s.name} />
                            {/each}
                        </ul>
                    {:else if statView === "hex"}
                        <div class="hexwrap">
                            <StatHex stats={pokemon.stats} color={accent} />
                        </div>
                    {/if}
                </div>
            {/key}
        {:else}
            <div class="empty">Press the button below to roll a Pokémon</div>
        {/if}

        <!-- «+N EXP» в случайных точках; несколько летят одновременно -->
        {#each floats as f (f.id)}
            <div
                class="exp-float"
                style:left="{f.x}%"
                style:top="{f.y}%"
                onanimationend={() => removeFloat(f.id)}
            >
                +{f.value} EXP
            </div>
        {/each}

        <!-- главное действие -->
        <button
            class="roll-fab"
            class:loading
            onclick={roll}
            disabled={loading}
            title="Roll"
        >
            {loading ? "…" : "ROLL"}
        </button>

        <!-- панель настроек поверх карты -->
        {#if settingsOpen}
            <div class="settings-panel">
                <div class="sp-head">
                    <span>Settings</span>
                    <button
                        class="sp-close"
                        onclick={() => (settingsOpen = false)}
                        aria-label="Close"
                    >
                        ✕
                    </button>
                </div>

                <div class="sp-group">
                    <span class="sp-label">Stats</span>
                    <div class="segmented">
                        {#each STAT_VIEWS as v}
                            <button class:active={statView === v} onclick={() => (statView = v)}>
                                {STAT_LABELS[v]}
                            </button>
                        {/each}
                    </div>
                </div>

                <div class="sp-group">
                    <span class="sp-label">Image</span>
                    <div class="segmented">
                        {#each IMAGE_MODES as m}
                            <button class:active={imageMode === m} onclick={() => (imageMode = m)}>
                                {IMAGE_LABELS[m]}
                            </button>
                        {/each}
                    </div>
                </div>

                <div class="sp-group">
                    <span class="sp-label">Window</span>
                    <label class="toggle">
                        <input type="checkbox" bind:checked={alwaysOnTop} />
                        <span>Always on top</span>
                    </label>
                </div>

                <!-- зерно показываем только для пиксель/ascii -->
                {#if imageMode === "pixel"}
                    <div class="sp-group">
                        <span class="sp-label">Pixels per row: {pixelCols}</span>
                        <input
                            class="slider"
                            type="range"
                            min="8"
                            max="64"
                            step="4"
                            bind:value={pixelCols}
                        />
                    </div>
                {:else if imageMode === "ascii"}
                    <div class="sp-group">
                        <span class="sp-label">Chars per row: {asciiCols}</span>
                        <input
                            class="slider"
                            type="range"
                            min="16"
                            max="72"
                            step="4"
                            bind:value={asciiCols}
                        />
                    </div>
                {/if}

                <div class="sp-group">
                    <span class="sp-label">Updates</span>
                    <label class="toggle">
                        <input type="checkbox" bind:checked={autoCheck} />
                        <span>Auto-check on start</span>
                    </label>
                    <button
                        class="update-btn ghost"
                        onclick={() => checkUpdate(true)}
                        disabled={checking || updating}
                    >
                        Check for updates
                    </button>
                </div>

                <div class="sp-footer">
                    <button
                        class="link-btn"
                        onclick={() => openUrl("https://github.com/vdzhagaev/pokeroll")}
                        title="GitHub"
                        aria-label="GitHub"
                    >
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path
                                d="M12 .5C5.37.5 0 5.87 0 12.5c0 5.3 3.44 9.8 8.21 11.39.6.11.82-.26.82-.58 0-.29-.01-1.05-.02-2.06-3.34.73-4.04-1.61-4.04-1.61-.55-1.39-1.34-1.76-1.34-1.76-1.09-.75.08-.74.08-.74 1.21.09 1.84 1.24 1.84 1.24 1.07 1.84 2.81 1.31 3.5 1 .11-.78.42-1.31.76-1.61-2.67-.3-5.47-1.34-5.47-5.95 0-1.31.47-2.38 1.24-3.22-.12-.3-.54-1.52.12-3.17 0 0 1.01-.32 3.3 1.23a11.5 11.5 0 0 1 6 0c2.29-1.55 3.3-1.23 3.3-1.23.66 1.65.24 2.87.12 3.17.77.84 1.24 1.91 1.24 3.22 0 4.62-2.81 5.64-5.49 5.94.43.37.81 1.1.81 2.22 0 1.6-.01 2.89-.01 3.29 0 .32.22.7.83.58A12.01 12.01 0 0 0 24 12.5C24 5.87 18.63.5 12 .5Z"
                            />
                        </svg>
                    </button>
                    <button
                        class="link-btn"
                        onclick={() => openUrl("mailto:slava.dzhagaev@proton.me")}
                        title="Email"
                        aria-label="Email"
                    >
                        <svg
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <rect x="3" y="5" width="18" height="14" rx="2" />
                            <path d="m3 7 9 6 9-6" />
                        </svg>
                    </button>
                </div>
            </div>
        {/if}

        <!-- модалка обновления: проверка / подтверждение / установка / результат -->
        {#if phase !== "idle"}
            <div class="update-modal" onmousedown={(e) => e.stopPropagation()}>
                <div class="um-box">
                    {#if phase === "checking" || phase === "installing"}
                        <div class="spinner"></div>
                    {/if}

                    <div class="um-title">
                        {#if phase === "checking"}
                            Checking for updates…
                        {:else if phase === "installing"}
                            Installing update…
                        {:else if phase === "available"}
                            Install update v{updateInfo?.version}?
                        {:else if phase === "uptodate"}
                            Already up to date
                        {:else}
                            Update failed
                        {/if}
                    </div>

                    {#if phase === "available" && updateInfo?.body}
                        <pre class="um-notes">{updateInfo.body}</pre>
                    {:else if phase === "error" && updateError}
                        <pre class="um-notes um-err">{updateError}</pre>
                    {/if}

                    <div class="um-actions">
                        {#if phase === "checking"}
                            <button class="update-btn ghost" onclick={cancelUpdate}>Cancel</button>
                        {:else if phase === "available"}
                            <button class="update-btn" onclick={installUpdate}>Install</button>
                            <button class="update-btn ghost" onclick={cancelUpdate}>Cancel</button>
                        {:else if phase === "uptodate" || phase === "error"}
                            <button class="update-btn ghost" onclick={() => (phase = "idle")}>
                                Close
                            </button>
                        {/if}
                    </div>
                </div>
            </div>
        {/if}
    </div>
</main>

<style>
    :global(html),
    :global(body) {
        margin: 0;
        height: 100%;
        background: transparent;
        overflow: hidden; /* виджет не скроллится */
    }

    :root {
        font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
        font-size: 16px;
        line-height: 24px;
        color: #0f0f0f;
        font-synthesis: none;
        text-rendering: optimizeLegibility;
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
    }

    /* виджет = карта на весь вебвью; поля прозрачные под тень и FAB */
    .container {
        box-sizing: border-box;
        height: 100vh;
        padding: 0; /* без прозрачных отступов вообще */
        display: flex;
        overflow: hidden;
        background: transparent;
    }

    .card {
        position: relative;
        flex: 1;
        display: flex;
        flex-direction: column;
        padding: 1.2em 1.4em 2em;
        --card-bg: #ffffff;
        border-radius: 8px;
        border: 3px solid transparent;
        /* градиентная рамка: фон по padding-box + градиент по border-box */
        background:
            linear-gradient(var(--card-bg), var(--card-bg)) padding-box,
            var(--border-gradient) border-box;
        box-shadow: 0 8px 28px rgba(0, 0, 0, 0.3);
    }

    /* угловые контролы (просто с отступом, не по центру угла) */
    .corner {
        position: absolute;
        z-index: 2;
    }
    .corner.tl {
        top: 10px;
        left: 10px;
    }
    .corner.tr {
        top: 10px;
        right: 10px;
        display: flex;
        gap: 6px;
    }

    .icon-btn {
        width: 36px;
        height: 36px;
        padding: 0;
        display: grid;
        place-items: center;
        border-radius: 50%;
        border: 2px solid color-mix(in srgb, var(--accent) 55%, transparent);
        background: color-mix(in srgb, var(--accent) 14%, var(--card-bg));
        color: color-mix(in srgb, var(--accent), #000 22%);
        cursor: pointer;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
        transition:
            transform 0.12s,
            background 0.2s;
    }
    .icon-btn svg {
        width: 18px;
        height: 18px;
        fill: none;
        stroke: currentColor;
        stroke-width: 2.2;
        stroke-linecap: round;
        stroke-linejoin: round;
    }
    .icon-btn:hover {
        transform: scale(1.12);
    }
    .icon-btn:active {
        transform: scale(0.95);
    }

    /* главная кнопка ролла: скруглённый прямоугольник, низ вровень с низом виджета */
    .roll-fab {
        position: absolute;
        bottom: 20px;
        left: 50%;
        transform: translate(-50%, 0);
        z-index: 3;
        height: 38px;
        min-width: 96px;
        padding: 0 1.5em;
        display: grid;
        place-items: center;
        border-radius: 12px;
        border: 3px solid var(--card-bg);
        background: linear-gradient(
            135deg,
            var(--accent),
            color-mix(in srgb, var(--accent), #000 28%)
        );
        color: #fff;
        font-weight: 800;
        font-size: 0.95em;
        letter-spacing: 0.12em;
        cursor: pointer;
        box-shadow: 0 4px 14px rgba(0, 0, 0, 0.38);
        transition:
            transform 0.22s cubic-bezier(0.34, 1.7, 0.5, 1),
            filter 0.2s;
    }
    /* hover: вся кнопка слегка трясётся (центрирование -50% сохраняем в кейфреймах) */
    .roll-fab:hover {
        animation: roll-shake 0.3s ease-in-out infinite;
    }
    @keyframes roll-shake {
        0%,
        100% {
            transform: translate(-50%, 0) rotate(0deg);
        }
        25% {
            transform: translate(-52%, 0) rotate(-3deg);
        }
        75% {
            transform: translate(-48%, 0) rotate(3deg);
        }
    }
    /* нажатие: тряску глушим, кнопка вдавливается вниз и пружинит обратно */
    .roll-fab:active {
        animation: none;
        transform: translate(-50%, 3px);
    }
    .roll-fab.loading {
        pointer-events: none;
        opacity: 0.85;
    }

    /* всплывашка «+N EXP»: сразу плавно вверх + долгое затухание, без прыжка */
    .exp-float {
        position: absolute;
        z-index: 4;
        font-weight: 800;
        font-size: 1.05em;
        letter-spacing: 0.02em;
        color: #ffd34d;
        text-shadow:
            0 0 1px rgba(0, 0, 0, 0.6),
            0 2px 6px rgba(0, 0, 0, 0.5);
        white-space: nowrap;
        pointer-events: none;
        animation: exp-float 2.2s ease-out forwards;
    }
    @keyframes exp-float {
        0% {
            opacity: 0;
            transform: translate(-50%, 0);
        }
        10% {
            opacity: 1;
        }
        100% {
            opacity: 0;
            transform: translate(-50%, -90px);
        }
    }

    .title {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5em;
        min-height: 30px;
        padding: 0 2.5em;
        margin-bottom: 0.4em;
    }

    .dexno {
        font-variant-numeric: tabular-nums;
        font-weight: 800;
        font-size: 0.85em;
        color: color-mix(in srgb, var(--accent), #000 30%);
    }

    .name {
        margin: 0;
        font-size: 1.15em;
        font-weight: 800;
        text-transform: capitalize;
    }

    .new-badge {
        font-size: 0.58em;
        line-height: 1;
        font-weight: 800;
        letter-spacing: 0.06em;
        color: #7a4a00;
        background: linear-gradient(#ffd774, #f5b642);
        padding: 0.28em 0.42em;
        border-radius: 5px;
    }

    .body {
        flex: 1;
        display: grid;
        grid-template-columns: 288px 1fr;
        gap: 1.2em;
        align-items: center;
    }
    .card.hide-stats .body {
        grid-template-columns: 1fr;
    }
    /* статов нет — места по вертикали больше, увеличиваем арт */
    .card.hide-stats .art {
        width: 352px;
        height: 352px;
    }

    .left {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.7em;
    }

    /* мягкий «свет» под спрайтом: по центру, плавно гаснет к краям без кольца */
    .art {
        position: relative;
        width: 288px;
        height: 288px;
        box-sizing: border-box;
        border-radius: 16px;
        display: grid;
        place-items: center;
        background: radial-gradient(
            circle at 50% 50%,
            color-mix(in srgb, var(--accent) 30%, transparent) 0%,
            color-mix(in srgb, var(--accent) 12%, transparent) 42%,
            transparent 72%
        );
    }

    .sprite {
        display: block;
        width: 100%;
        height: 100%;
        object-fit: contain;
        image-rendering: auto;
        opacity: 0;
        transform: scale(0.85);
        transition:
            opacity 0.35s ease,
            transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
    }
    .sprite.loaded {
        opacity: 1;
        transform: scale(1);
    }

    .skeleton {
        position: absolute;
        inset: 16%;
        border-radius: 20px;
        background: color-mix(in srgb, var(--accent) 22%, var(--card-bg));
        animation: pulse 1s ease-in-out infinite;
    }
    @keyframes pulse {
        0%,
        100% {
            opacity: 0.35;
        }
        50% {
            opacity: 0.85;
        }
    }

    .types {
        display: flex;
        gap: 0.4em;
        justify-content: center;
    }

    .type {
        text-transform: capitalize;
        font-size: 0.85em;
        font-weight: 600;
        padding: 0.2em 0.8em;
        border-radius: 999px;
        color: #fff;
        text-shadow: 0 1px 1px rgba(0, 0, 0, 0.25);
    }

    .meta {
        display: flex;
        gap: 1.1em;
        font-size: 0.8em;
        opacity: 0.7;
    }
    .meta b {
        font-weight: 700;
    }

    .stats {
        list-style: none;
        text-align: left;
        padding: 0;
        margin: 0;
    }

    .hexwrap {
        display: grid;
        place-items: center;
        padding: 0.5em 0;
    }

    .empty {
        flex: 1;
        display: grid;
        place-items: center;
        opacity: 0.6;
        text-align: center;
        padding: 0 2em;
    }

    /* панель настроек поверх карты */
    .settings-panel {
        position: absolute;
        inset: 0;
        z-index: 5;
        border-radius: 6px;
        padding: 1.4em 1.6em;
        display: flex;
        flex-direction: column;
        gap: 1.2em;
        background: color-mix(in srgb, var(--card-bg) 90%, transparent);
        backdrop-filter: blur(7px);
        -webkit-backdrop-filter: blur(7px);
    }

    .sp-head {
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-weight: 800;
    }
    .sp-close {
        border: none;
        background: none;
        color: inherit;
        font-size: 1em;
        cursor: pointer;
        opacity: 0.55;
    }
    .sp-close:hover {
        opacity: 1;
    }

    .sp-group {
        display: flex;
        flex-direction: column;
        gap: 0.5em;
    }
    .sp-label {
        font-size: 0.8em;
        font-weight: 700;
        opacity: 0.6;
    }

    .segmented {
        display: flex;
        gap: 0.3em;
        padding: 0.25em;
        border-radius: 10px;
        background: color-mix(in srgb, var(--accent) 10%, transparent);
    }
    .segmented button {
        flex: 1;
        padding: 0.5em 0.4em;
        border: none;
        border-radius: 7px;
        background: transparent;
        color: inherit;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.15s;
    }
    .segmented button:hover {
        background: color-mix(in srgb, var(--accent) 14%, transparent);
    }
    .segmented button.active {
        background: var(--accent);
        color: #fff;
    }

    /* ползунок зерна — трек серый, заполнение/бегунок в акцент */
    .slider {
        width: 100%;
        height: 22px;
        margin: 0;
        cursor: pointer;
        accent-color: var(--accent);
    }

    .toggle {
        display: flex;
        align-items: center;
        gap: 0.5em;
        cursor: pointer;
        font-weight: 600;
    }
    .toggle input {
        width: 16px;
        height: 16px;
        cursor: pointer;
        accent-color: var(--accent);
    }

    /* футер настроек: ссылки на гитхаб/почту */
    .sp-footer {
        margin-top: auto; /* прижать к низу панели */
        display: flex;
        justify-content: center;
        gap: 0.8em;
        padding-top: 0.4em;
    }
    .link-btn {
        width: 34px;
        height: 34px;
        display: grid;
        place-items: center;
        border-radius: 8px;
        border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
        background: color-mix(in srgb, var(--accent) 10%, transparent);
        color: inherit;
        cursor: pointer;
        transition:
            background 0.15s,
            transform 0.12s;
    }
    .link-btn:hover {
        background: color-mix(in srgb, var(--accent) 22%, transparent);
        transform: translateY(-1px);
    }
    .link-btn svg {
        width: 18px;
        height: 18px;
    }

    /* модалка обновления */
    .update-modal {
        position: absolute;
        inset: 0;
        z-index: 6;
        display: grid;
        place-items: center;
        padding: 1.4em;
        border-radius: 8px;
        background: color-mix(in srgb, #000 48%, transparent);
        backdrop-filter: blur(4px);
        -webkit-backdrop-filter: blur(4px);
    }
    .um-box {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1em;
        max-width: 88%;
        padding: 1.5em 1.6em;
        border-radius: 12px;
        background: var(--card-bg);
        border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
        box-shadow: 0 12px 34px rgba(0, 0, 0, 0.45);
        text-align: center;
    }
    .um-title {
        font-weight: 800;
        font-size: 1.02em;
    }
    .um-notes {
        margin: 0;
        max-width: 100%;
        max-height: 160px;
        overflow: auto;
        padding: 0.7em 0.9em;
        border-radius: 8px;
        background: color-mix(in srgb, var(--accent) 8%, transparent);
        font-family: inherit;
        font-size: 0.82em;
        line-height: 1.4;
        white-space: pre-wrap;
        text-align: left;
        opacity: 0.9;
    }
    .um-err {
        background: color-mix(in srgb, #e5484d 14%, transparent);
        color: #e5484d;
    }
    .um-actions {
        display: flex;
        gap: 0.6em;
    }
    .spinner {
        width: 34px;
        height: 34px;
        border-radius: 50%;
        border: 3px solid color-mix(in srgb, var(--accent) 22%, transparent);
        border-top-color: var(--accent);
        animation: spin 0.8s linear infinite;
    }
    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }
    .update-btn {
        padding: 0.5em 1em;
        border-radius: 8px;
        border: none;
        background: var(--accent);
        color: #fff;
        font-weight: 700;
        cursor: pointer;
        transition: filter 0.15s;
    }
    .update-btn:hover {
        filter: brightness(1.08);
    }
    .update-btn:disabled {
        opacity: 0.6;
        cursor: default;
    }
    .update-btn.ghost {
        background: color-mix(in srgb, var(--accent) 14%, transparent);
        color: inherit;
        border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
    }

    @media (prefers-color-scheme: dark) {
        :root {
            color: #f6f6f6;
        }
        .card {
            --card-bg: #1f1f1f;
        }
    }
</style>
