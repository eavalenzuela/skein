<script lang="ts">
  import { onMount } from "svelte";
  import {
    tweaks,
    persist,
    type Theme,
    type ShelfStyle,
    type SidebarMode,
    type PageFont,
  } from "../tweaks.svelte.js";
  import { embedderState, downloadModel } from "../embedder.svelte.js";
  import {
    vaultState,
    open as openVaultPath,
    openFromArchive,
    close as closeVault,
    refreshVault,
  } from "../vault.svelte.js";
  import {
    exportVault,
    listTrash,
    emptyTrash,
    restoreTrashedPage,
    type TrashEntry,
  } from "../vault.js";
  import { toastSuccess } from "../toasts.svelte.js";
  import { settingsState, setAutoTag } from "../settingsUi.svelte.js";
  import {
    hasSecret,
    setSecret,
    clearSecret,
    getSettings,
    setSettings,
    gitStatus,
    gitSetRemote,
    gitPull,
    gitPush,
    gitCommitAll,
    type SecretName,
    type GitStatus,
    type GitPullResult,
  } from "../settings.js";
  import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
  import { focusTrap } from "../focusTrap.js";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();

  const themes: Theme[] = ["dark", "light"];
  const shelves: ShelfStyle[] = ["abstract", "suggestive", "tactile"];
  const sidebars: SidebarMode[] = ["open", "collapsed", "hidden"];
  const fonts: PageFont[] = ["Source Serif 4", "Iowan Old Style", "Spectral", "Lora"];

  let anthropicSet = $state(false);
  let voyageSet = $state(false);
  let anthropicInput = $state("");
  let voyageInput = $state("");
  let savingAnthropic = $state(false);
  let savingVoyage = $state(false);
  let secretError = $state<string | null>(null);

  let dailyBook = $state("Daily");
  let dailyTemplate = $state(
    `---\ntitle: {{long_date}}\ntags: [daily]\ncreated: {{date}}\n---\n\n## Morning\n\n\n## Notes\n\n\n## Tomorrow\n\n`,
  );
  let dailyReminderTime = $state("");
  let dailyError = $state<string | null>(null);
  let dailySaving = $state(false);

  async function loadDaily() {
    try {
      const s = await getSettings();
      if (s.daily_book) dailyBook = s.daily_book;
      if (s.daily_template) dailyTemplate = s.daily_template;
      if (s.daily_reminder_time) dailyReminderTime = s.daily_reminder_time;
      markDailyClean();
    } catch (e) {
      dailyError = String(e);
    }
  }

  // The Appearance section applies instantly, but Daily notes and Sync need
  // an explicit save. Closing the modal used to throw those edits away
  // silently — an Escape press, trained by the palette, could lose a git
  // remote or a rewritten template. Track a baseline and confirm instead.
  let dailyBaseline = $state("");
  let gitBaseline = $state("");

  const dailySnapshot = () => JSON.stringify([dailyBook, dailyTemplate, dailyReminderTime]);
  const gitSnapshot = () => JSON.stringify([gitRemote, gitBranch, gitAuthKind, gitCommitMsg]);

  function markDailyClean() {
    dailyBaseline = dailySnapshot();
  }
  function markGitClean() {
    gitBaseline = gitSnapshot();
  }

  let dailyDirty = $derived(dailyBaseline !== "" && dailySnapshot() !== dailyBaseline);
  let gitDirty = $derived(gitBaseline !== "" && gitSnapshot() !== gitBaseline);
  let dirtySections = $derived(
    [dailyDirty ? "Daily notes" : null, gitDirty ? "Sync" : null].filter(
      (s): s is string => s !== null,
    ),
  );

  function requestClose() {
    if (dirtySections.length === 0) {
      onClose();
      return;
    }
    confirmingClose = true;
  }
  let confirmingClose = $state(false);

  async function saveDaily() {
    dailySaving = true;
    dailyError = null;
    try {
      // If a reminder time is set and we don't yet have notification
      // permission, request it now.
      if (dailyReminderTime) {
        const granted = await isPermissionGranted();
        if (!granted) {
          const decision = await requestPermission();
          if (decision !== "granted") {
            dailyError = "Notification permission was not granted; reminders won't fire.";
          }
        }
      }
      await setSettings({
        daily_book: dailyBook.trim() || "Daily",
        daily_template: dailyTemplate,
        daily_reminder_time: dailyReminderTime.trim(),
      });
      markDailyClean();
    } catch (e) {
      dailyError = String(e);
    } finally {
      dailySaving = false;
    }
  }

  async function refreshSecrets() {
    try {
      [anthropicSet, voyageSet, gitTokenSet] = await Promise.all([
        hasSecret("anthropic_api_key"),
        hasSecret("voyage_api_key"),
        hasSecret("git_token"),
      ]);
    } catch (e) {
      secretError = String(e);
    }
  }

  // --- Sync (Phase 13) ---
  let gitTokenSet = $state(false);
  let gitTokenInput = $state("");
  let gitRemote = $state("");
  let gitBranch = $state("main");
  let gitAuthKind = $state<"none" | "token" | "ssh-agent">("none");
  let gitStatusData = $state<GitStatus | null>(null);
  let gitStatusFetchedAt = $state<number | null>(null);
  let gitStatusAgeNow = $state(Date.now());
  let gitBusy = $state(false);
  let gitMessage = $state<string | null>(null);
  let gitError = $state<string | null>(null);
  let gitCommitMsg = $state("");

  // Tick once a second so the "fetched Ns ago" label updates while the
  // modal is open. Cleared in onDestroy via the effect's return.
  $effect(() => {
    const t = setInterval(() => {
      gitStatusAgeNow = Date.now();
    }, 1000);
    return () => clearInterval(t);
  });

  function gitFreshnessLabel(): string {
    if (gitStatusFetchedAt == null) return "not fetched yet";
    const sec = Math.max(0, Math.round((gitStatusAgeNow - gitStatusFetchedAt) / 1000));
    if (sec < 5) return "just now";
    if (sec < 60) return `${sec}s ago`;
    const min = Math.round(sec / 60);
    if (min < 60) return `${min}m ago`;
    const hr = Math.round(min / 60);
    return `${hr}h ago`;
  }

  async function loadGitConfig() {
    try {
      const s = await getSettings();
      if (s.git_remote_url) gitRemote = s.git_remote_url;
      if (s.git_branch) gitBranch = s.git_branch;
      if (s.git_auth_kind) {
        gitAuthKind = (s.git_auth_kind as typeof gitAuthKind) || "none";
      }
      markGitClean();
    } catch (e) {
      gitError = String(e);
    }
  }

  async function refreshGitStatus() {
    try {
      gitStatusData = await gitStatus();
      gitStatusFetchedAt = Date.now();
      gitStatusAgeNow = gitStatusFetchedAt;
    } catch (e) {
      gitError = String(e);
    }
  }

  async function saveGitConfig() {
    gitBusy = true;
    gitError = null;
    gitMessage = null;
    try {
      await setSettings({
        git_remote_url: gitRemote.trim(),
        git_branch: gitBranch.trim() || "main",
        git_auth_kind: gitAuthKind,
      });
      if (gitRemote.trim()) {
        await gitSetRemote(gitRemote.trim());
      }
      if (gitTokenInput && gitAuthKind === "token") {
        await setSecret("git_token", gitTokenInput);
        gitTokenInput = "";
        gitTokenSet = true;
      }
      await refreshGitStatus();
      gitMessage = "Saved.";
      markGitClean();
    } catch (e) {
      gitError = String(e);
    } finally {
      gitBusy = false;
    }
  }

  async function runPull() {
    gitBusy = true;
    gitError = null;
    gitMessage = null;
    try {
      const r: GitPullResult = await gitPull();
      gitMessage =
        r.kind === "conflicts"
          ? `Pulled with conflicts: ${r.conflicted.join(", ")}`
          : `Pull: ${r.kind}.`;
      await refreshGitStatus();
    } catch (e) {
      gitError = String(e);
    } finally {
      gitBusy = false;
    }
  }

  async function runPush() {
    gitBusy = true;
    gitError = null;
    gitMessage = null;
    try {
      // Auto-commit any dirty files before pushing so users don't see a
      // silent "nothing pushed" when they had unsaved changes.
      const msg = gitCommitMsg.trim() || "Skein update";
      const committed = await gitCommitAll(msg);
      await gitPush();
      gitMessage = committed ? `Pushed (committed: ${msg}).` : "Pushed.";
      gitCommitMsg = "";
      await refreshGitStatus();
    } catch (e) {
      gitError = String(e);
    } finally {
      gitBusy = false;
    }
  }

  async function clearGitToken() {
    try {
      await clearSecret("git_token");
      gitTokenSet = false;
    } catch (e) {
      gitError = String(e);
    }
  }

  onMount(() => {
    void refreshSecrets();
    void loadDaily();
    void loadGitConfig();
    void refreshGitStatus();
    void loadTrash();
  });

  // --- Privacy -------------------------------------------------------
  let autoTagError = $state<string | null>(null);

  async function toggleAutoTag(on: boolean) {
    autoTagError = null;
    try {
      await setAutoTag(on);
    } catch (e) {
      autoTagError = String(e);
      settingsState.autoTag = !on;
    }
  }

  // --- Trash ---------------------------------------------------------
  let trashItems = $state<TrashEntry[]>([]);
  let trashBusy = $state(false);
  let trashError = $state<string | null>(null);

  async function loadTrash() {
    if (!vaultState.vault) return;
    try {
      trashItems = await listTrash();
    } catch (e) {
      trashError = String(e);
    }
  }

  async function restoreOne(id: string) {
    trashBusy = true;
    trashError = null;
    try {
      const rel = await restoreTrashedPage(id);
      await refreshVault();
      await loadTrash();
      toastSuccess(`Restored ${rel}`);
    } catch (e) {
      trashError = String(e);
    } finally {
      trashBusy = false;
    }
  }

  async function emptyAll() {
    trashBusy = true;
    trashError = null;
    try {
      const n = await emptyTrash();
      await loadTrash();
      toastSuccess(`Emptied the trash`, `${n} ${n === 1 ? "page" : "pages"} removed for good`);
    } catch (e) {
      trashError = String(e);
    } finally {
      trashBusy = false;
    }
  }

  async function saveSecret(name: SecretName, value: string) {
    secretError = null;
    if (name === "anthropic_api_key") savingAnthropic = true;
    else savingVoyage = true;
    try {
      await setSecret(name, value);
      if (name === "anthropic_api_key") {
        anthropicInput = "";
        anthropicSet = true;
      } else {
        voyageInput = "";
        voyageSet = true;
      }
    } catch (e) {
      secretError = String(e);
    } finally {
      savingAnthropic = false;
      savingVoyage = false;
    }
  }

  async function clearAndForget(name: SecretName) {
    secretError = null;
    try {
      await clearSecret(name);
      if (name === "anthropic_api_key") anthropicSet = false;
      else voyageSet = false;
    } catch (e) {
      secretError = String(e);
    }
  }

  async function pickNewVault() {
    const selected = await openDialog({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await openVaultPath(selected);
    }
  }

  let restoring = $state(false);
  let restoreError = $state<string | null>(null);
  async function restoreFromArchive() {
    restoreError = null;
    const archive = await openDialog({
      multiple: false,
      filters: [{ name: "Zip archive", extensions: ["zip"] }],
    });
    if (typeof archive !== "string") return;
    const dest = await openDialog({ directory: true, multiple: false });
    if (typeof dest !== "string") return;
    restoring = true;
    try {
      await openFromArchive(archive, dest);
      onClose();
    } catch (e) {
      restoreError = String(e);
    } finally {
      restoring = false;
    }
  }

  let exporting = $state(false);
  let exportError = $state<string | null>(null);
  let exportedTo = $state<string | null>(null);
  async function exportCurrentVault() {
    if (!vaultState.vault) return;
    exportError = null;
    exportedTo = null;
    const defaultName = `${vaultState.vault.name}.zip`;
    const dest = await saveDialog({
      defaultPath: defaultName,
      filters: [{ name: "Zip archive", extensions: ["zip"] }],
    });
    if (typeof dest !== "string") return;
    exporting = true;
    try {
      await exportVault(dest);
      exportedTo = dest;
    } catch (e) {
      exportError = String(e);
    } finally {
      exporting = false;
    }
  }
</script>

<!-- Escape is bound at the window, not the overlay: clicking a button that
     then disables itself (save, pull, push) moves focus to <body>, and a
     handler on the overlay never sees the keypress — leaving the modal
     unclosable from the keyboard. -->
<svelte:window
  onkeydown={(e) => {
    if (e.key !== "Escape") return;
    e.preventDefault();
    if (confirmingClose) confirmingClose = false;
    else requestClose();
  }}
/>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={requestClose}>
  <div
    class="modal"
    role="dialog"
    aria-label="Settings"
    aria-modal="true"
    tabindex="-1"
    use:focusTrap
    onclick={(e) => e.stopPropagation()}
  >
    <header>
      <h2>Settings</h2>
      <button class="close" onclick={requestClose} aria-label="Close">×</button>
    </header>

    <div class="body">
      <section>
        <h3>Vault</h3>
        {#if vaultState.vault}
          <div class="row">
            <div class="kv">
              <div class="k">{vaultState.vault.name}</div>
              <div class="v mono">{vaultState.vault.root}</div>
            </div>
            <div class="actions">
              <button onclick={pickNewVault}>change…</button>
              <!-- Named for what it does: a bare "close" next to the dialog's
                   own × reads as "close this dialog", and styling it as a
                   danger button made that misread destructive-looking. -->
              <button onclick={closeVault}>close vault</button>
            </div>
          </div>
          <div class="row">
            <div class="kv">
              <div class="k">Export</div>
              <div class="v muted">
                Bundles vault contents + embeddings sidecar into a zip.
              </div>
            </div>
            <div class="actions">
              <button onclick={exportCurrentVault} disabled={exporting}>
                {exporting ? "exporting…" : "export vault…"}
              </button>
            </div>
          </div>
          {#if exportedTo}
            <p class="muted mono">Wrote {exportedTo}</p>
          {/if}
          {#if exportError}
            <p class="danger">{exportError}</p>
          {/if}
        {:else}
          <div class="row">
            <p class="muted">No vault open.</p>
            <div class="actions">
              <button class="primary" onclick={pickNewVault}>pick a folder</button>
              <button onclick={restoreFromArchive} disabled={restoring}>
                {restoring ? "restoring…" : "open from archive…"}
              </button>
            </div>
          </div>
          <p class="muted">
            To import an Obsidian vault or a plain folder of markdown, just
            pick the folder — the existing structure (subdirs as books,
            top-level <code>.md</code> as loose pages) carries over with no
            transformation.
          </p>
          {#if restoreError}
            <p class="danger">{restoreError}</p>
          {/if}
        {/if}
      </section>

      <section>
        <h3>Privacy</h3>
        <p class="muted">
          Skein is local-first: your vault, index and embeddings never leave
          this machine. Two features do send text to Anthropic — the chat
          sidebar, when you send a message, and auto-tagging, below.
        </p>
        <div class="row">
          <div class="kv">
            <div class="k">Auto-suggest tags</div>
            <div class="v muted wrap">
              Sends a page's title and text (up to 6 KB) to Anthropic a few
              seconds after you stop typing, and bills your API key. Off by
              default.
            </div>
          </div>
          <div class="actions">
            <label class="opt">
              <input
                type="checkbox"
                checked={settingsState.autoTag}
                onchange={(e) => void toggleAutoTag(e.currentTarget.checked)}
              />
              {settingsState.autoTag ? "on" : "off"}
            </label>
          </div>
        </div>
        {#if autoTagError}
          <p class="danger">{autoTagError}</p>
        {/if}
      </section>

      {#if vaultState.vault}
        <section>
          <h3>Trash</h3>
          {#if trashItems.length === 0}
            <p class="muted">
              Nothing in the trash. Deleted pages land here and are removed
              for good after 30 days.
            </p>
          {:else}
            <ul class="trash-list">
              {#each trashItems as t (t.id)}
                <li>
                  <div class="kv">
                    <div class="k">{t.title}</div>
                    <div class="v mono">{t.rel_path}</div>
                  </div>
                  <button onclick={() => void restoreOne(t.id)} disabled={trashBusy}>
                    restore
                  </button>
                </li>
              {/each}
            </ul>
            <div class="row">
              <p class="muted">
                {trashItems.length}
                {trashItems.length === 1 ? "page" : "pages"} recoverable
              </p>
              <div class="actions">
                <button class="danger" onclick={() => void emptyAll()} disabled={trashBusy}>
                  empty trash
                </button>
              </div>
            </div>
          {/if}
          {#if trashError}
            <p class="danger">{trashError}</p>
          {/if}
        </section>

        <section>
          <h3>Sync</h3>
          <div class="grid">
            <div class="grid-label">Remote URL</div>
            <input
              type="text"
              bind:value={gitRemote}
              placeholder="git@github.com:you/notes.git or https://github.com/you/notes.git"
            />
            <div class="grid-label">Branch</div>
            <input type="text" bind:value={gitBranch} placeholder="main" />
            <div class="grid-label">Commit message</div>
            <input
              type="text"
              bind:value={gitCommitMsg}
              placeholder="used when pushing (optional)"
            />
            <div class="grid-label">Auth</div>
            <div class="radios">
              <label class="opt">
                <input type="radio" bind:group={gitAuthKind} value="none" /> none
              </label>
              <label class="opt">
                <input type="radio" bind:group={gitAuthKind} value="token" /> HTTPS token
              </label>
              <label class="opt">
                <input type="radio" bind:group={gitAuthKind} value="ssh-agent" /> ssh-agent
              </label>
            </div>
            {#if gitAuthKind === "token"}
              <div class="grid-label">Token</div>
              <div class="row">
                {#if gitTokenSet}
                  <span class="muted">stored in keychain</span>
                  <button onclick={clearGitToken}>clear</button>
                {:else}
                  <input
                    type="password"
                    bind:value={gitTokenInput}
                    placeholder="paste personal access token"
                  />
                {/if}
              </div>
            {/if}
          </div>
          <div class="row">
            <div class="actions">
              <button onclick={saveGitConfig} disabled={gitBusy}>save</button>
              <button onclick={refreshGitStatus} disabled={gitBusy}>refresh status</button>
              <button onclick={runPull} disabled={gitBusy}>pull</button>
              <button onclick={runPush} disabled={gitBusy}>push</button>
            </div>
          </div>

          {#if gitStatusData}
            <div class="kv">
              <div class="v muted git-fresh">
                Status fetched <span class="git-age">{gitFreshnessLabel()}</span>
              </div>
              {#if !gitStatusData.initialized}
                <div class="v muted">Not a git repo yet — enter a remote URL and save.</div>
              {:else}
                <div class="v">
                  <span class="mono">{gitStatusData.branch ?? "(detached)"}</span>
                  {#if gitStatusData.ahead || gitStatusData.behind}
                    — ahead {gitStatusData.ahead}, behind {gitStatusData.behind}
                  {/if}
                </div>
                {#if gitStatusData.remote_url}
                  <div class="v muted mono">{gitStatusData.remote_url}</div>
                {/if}
                {#if gitStatusData.conflicted.length}
                  <div class="v danger">
                    Conflicts ({gitStatusData.conflicted.length}):
                    {gitStatusData.conflicted.join(", ")}
                  </div>
                {/if}
                {#if gitStatusData.dirty.length}
                  <div class="v muted">
                    Dirty ({gitStatusData.dirty.length}):
                    {gitStatusData.dirty
                      .slice(0, 8)
                      .map((d) => `${d.state} ${d.path}`)
                      .join(", ")}{gitStatusData.dirty.length > 8 ? "…" : ""}
                  </div>
                {/if}
              {/if}
            </div>
          {/if}
          {#if gitMessage}<p class="muted">{gitMessage}</p>{/if}
          {#if gitError}<p class="danger">{gitError}</p>{/if}
        </section>
      {/if}

      <section>
        <h3>Appearance</h3>
        <div class="grid">
          <div class="grid-label">Theme</div>
          <div class="radios">
            {#each themes as t (t)}
              <label class="opt">
                <input type="radio" bind:group={tweaks.theme} value={t} onchange={persist} />
                {t}
              </label>
            {/each}
          </div>

          <div class="grid-label">Shelf realism</div>
          <div class="radios">
            {#each shelves as s (s)}
              <label class="opt">
                <input type="radio" bind:group={tweaks.shelfStyle} value={s} onchange={persist} />
                {s}
              </label>
            {/each}
          </div>

          <div class="grid-label">Sidebar default</div>
          <div class="radios">
            {#each sidebars as s (s)}
              <label class="opt">
                <input type="radio" bind:group={tweaks.sidebar} value={s} onchange={persist} />
                {s}
              </label>
            {/each}
          </div>

          <div class="grid-label">Page serif</div>
          <select bind:value={tweaks.pageFont} onchange={persist}>
            {#each fonts as f (f)}
              <option value={f}>{f}</option>
            {/each}
          </select>
        </div>
      </section>

      <section>
        <h3>Embeddings</h3>
        <div class="row">
          <div class="kv">
            <div class="k">
              {embedderState.status?.local
                ? "BGE-small-en-v1.5 (local ONNX)"
                : "hash-bag (fallback)"}
            </div>
            <div class="v">
              {embedderState.status?.local
                ? "Semantic similarity is on. Pages share neighbours by meaning."
                : "Keyword-overlap fallback. Download the local model for semantic neighbours."}
            </div>
          </div>
          {#if !embedderState.status?.local}
            <button class="primary" onclick={downloadModel} disabled={embedderState.busy}>
              {embedderState.busy ? "downloading…" : "download BGE-small (~130 MB)"}
            </button>
          {/if}
        </div>
        {#if embedderState.busy}
          <div class="progress" role="progressbar" aria-label="Downloading embedding model">
            <div class="progress-bar"></div>
          </div>
          <p class="muted progress-hint">
            Streaming the model from Hugging Face — first run only. Skein stays usable; the
            keyword-overlap fallback handles "Related" until the model is ready.
          </p>
        {/if}
        {#if embedderState.error}
          <p class="error">{embedderState.error}</p>
        {/if}
      </section>

      <section>
        <h3>Daily notes</h3>
        <div class="grid">
          <div class="grid-label">Book name</div>
          <input class="text-input" bind:value={dailyBook} placeholder="Daily" />

          <div class="grid-label">Reminder time</div>
          <div class="reminder-row">
            <input
              class="text-input small"
              bind:value={dailyReminderTime}
              placeholder="HH:MM (24h)"
            />
            <span class="muted">leave blank to disable</span>
          </div>

          <div class="grid-label">Template</div>
          <textarea bind:value={dailyTemplate} rows="9" spellcheck="false"></textarea>

          <div class="grid-label"></div>
          <div class="row">
            <span class="muted"
              >Placeholders: {`{{date}}, {{long_date}}, {{weekday}}, {{time}}`}</span
            >
            <button class="primary" onclick={saveDaily} disabled={dailySaving}>
              {dailySaving ? "saving…" : "save daily settings"}
            </button>
          </div>
        </div>
        {#if dailyError}
          <p class="error">{dailyError}</p>
        {/if}
      </section>

      <section>
        <h3>API keys</h3>
        <p class="muted">
          Stored in your OS keychain — libsecret on Linux, Credential Manager on Windows. Skein
          never writes them to disk in plaintext and never returns them to the UI after they're
          saved.
        </p>

        <div class="key">
          <div class="key-info">
            <span class="k">Anthropic</span>
            <span class="v">Unlocks the chat sidebar and auto-tagging.</span>
          </div>
          <div class="key-input">
            {#if anthropicSet && !anthropicInput}
              <input type="text" disabled value="•••• configured ••••" />
              <button class="danger" onclick={() => clearAndForget("anthropic_api_key")}
                >forget</button
              >
            {:else}
              <input
                type="password"
                placeholder={anthropicSet ? "enter a new key to replace" : "sk-ant-…"}
                bind:value={anthropicInput}
                autocomplete="off"
                spellcheck="false"
              />
              <button
                class="primary"
                onclick={() => saveSecret("anthropic_api_key", anthropicInput)}
                disabled={!anthropicInput || savingAnthropic}
              >
                {savingAnthropic ? "saving…" : "save"}
              </button>
            {/if}
          </div>
        </div>

        <div class="key">
          <div class="key-info">
            <span class="k">Voyage</span>
            <span class="v">Optional. Higher-quality remote embeddings (Phase 5c).</span>
          </div>
          <div class="key-input">
            {#if voyageSet && !voyageInput}
              <input type="text" disabled value="•••• configured ••••" />
              <button class="danger" onclick={() => clearAndForget("voyage_api_key")}>forget</button
              >
            {:else}
              <input
                type="password"
                placeholder={voyageSet ? "enter a new key to replace" : "pa-…"}
                bind:value={voyageInput}
                autocomplete="off"
                spellcheck="false"
              />
              <button
                class="primary"
                onclick={() => saveSecret("voyage_api_key", voyageInput)}
                disabled={!voyageInput || savingVoyage}
              >
                {savingVoyage ? "saving…" : "save"}
              </button>
            {/if}
          </div>
        </div>

        {#if secretError}
          <p class="error">{secretError}</p>
        {/if}
      </section>
    </div>
  </div>

  {#if confirmingClose}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="confirm-scrim" onclick={(e) => e.stopPropagation()}>
      <div
        class="confirm"
        role="alertdialog"
        aria-modal="true"
        aria-label="Unsaved settings"
        tabindex="-1"
        use:focusTrap
      >
        <h3>Unsaved changes</h3>
        <p>
          {dirtySections.join(" and ")}
          {dirtySections.length === 1 ? "has" : "have"} edits that haven't been saved. Closing now discards
          {dirtySections.length === 1 ? "them" : "them"}.
        </p>
        <div class="actions">
          <button onclick={() => (confirmingClose = false)}>Keep editing</button>
          <button
            class="danger"
            onclick={() => {
              confirmingClose = false;
              onClose();
            }}>Discard</button
          >
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .confirm-scrim {
    position: absolute;
    inset: 0;
    background: oklch(0 0 0 / 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
  }
  .confirm {
    width: min(380px, 90%);
    background: var(--chrome);
    border: 1px solid var(--chrome-edge);
    border-radius: 10px;
    padding: 18px;
    box-shadow: 0 20px 50px -18px oklch(0 0 0 / 0.7);
    outline: none;
  }
  .confirm h3 {
    margin: 0 0 6px;
    font-family: var(--page-font, "Source Serif 4"), serif;
    font-size: 16px;
    font-weight: 500;
    color: var(--ink);
    text-transform: none;
    letter-spacing: normal;
  }
  .confirm p {
    margin: 0 0 14px;
    font-size: 12.5px;
    color: var(--ink-3);
    line-height: 1.45;
  }
  .confirm .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .overlay {
    position: fixed;
    inset: 0;
    background: oklch(0 0 0 / 0.72);
    z-index: 800;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 6vh;
    backdrop-filter: blur(3px);
  }
  .modal {
    width: 720px;
    max-width: 92vw;
    max-height: 86vh;
    background: var(--chrome-2);
    border: 1px solid var(--chrome-edge);
    border-radius: 10px;
    color: var(--ink);
    font-family: "Inter", system-ui, sans-serif;
    box-shadow: 0 30px 80px -20px oklch(0 0 0 / 0.55);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 20px;
    border-bottom: 1px solid var(--chrome-edge);
    background: var(--chrome);
  }
  h2 {
    margin: 0;
    font-family: "Source Serif 4", Georgia, serif;
    font-weight: 600;
    font-size: 18px;
  }
  .close {
    width: 26px;
    height: 26px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    color: var(--ink-3);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
  }
  .close:hover {
    background: oklch(1 0 0 / 0.06);
    color: var(--ink);
  }
  .body {
    overflow: auto;
    padding: 12px 20px 20px;
  }
  section {
    padding: 16px 0;
    border-bottom: 1px solid var(--chrome-edge);
  }
  section:last-child {
    border-bottom: 0;
  }
  h3 {
    margin: 0 0 12px;
    font-size: 11.5px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--ink-3);
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .kv {
    flex: 1;
    min-width: 0;
  }
  .kv .k {
    font-size: 13px;
    color: var(--ink);
    margin-bottom: 2px;
  }
  .kv .v {
    font-size: 11.5px;
    color: var(--ink-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .kv .v.mono {
    font-family: "JetBrains Mono", monospace;
    font-size: 11px;
  }
  /* `.v` truncates to one line for paths; explanatory copy needs to wrap. */
  .kv .v.wrap {
    white-space: normal;
    line-height: 1.45;
    max-width: 62ch;
  }
  .actions {
    display: flex;
    gap: 6px;
  }
  .grid {
    display: grid;
    grid-template-columns: 140px 1fr;
    column-gap: 16px;
    row-gap: 10px;
    align-items: center;
    font-size: 12px;
  }
  .grid > .grid-label {
    color: var(--ink-3);
    font-size: 11.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .radios {
    display: flex;
    gap: 14px;
    flex-wrap: wrap;
  }
  .opt {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    font-size: 12px;
    color: var(--ink-2);
  }
  input[type="radio"] {
    accent-color: var(--accent);
  }
  select {
    padding: 5px 8px;
    background: oklch(from var(--chrome-2) calc(l + 0.03) c h);
    color: var(--ink);
    border: 1px solid var(--chrome-edge);
    border-radius: 5px;
    font-size: 12px;
    width: max-content;
    min-width: 200px;
  }
  /* Base styling for every free-text field in the modal. The Sync section's
     inputs previously carried no class and rendered as native white boxes
     inside the dark chrome; styling by type means a new field can't drift. */
  .body input[type="text"],
  .body input[type="password"],
  .text-input {
    padding: 6px 10px;
    background: oklch(from var(--chrome-2) calc(l + 0.03) c h);
    border: 1px solid var(--chrome-edge);
    color: var(--ink);
    border-radius: 5px;
    font-family: "Inter", system-ui, sans-serif;
    font-size: 12px;
    outline: none;
    width: 240px;
  }
  /* The Sync grid gives its fields a full column. */
  .grid > input[type="text"] {
    width: 100%;
  }
  .body input[type="text"]:focus,
  .body input[type="password"]:focus {
    border-color: var(--accent-edge);
  }
  .text-input.small {
    width: 120px;
    font-family: "JetBrains Mono", monospace;
  }
  .text-input:focus {
    border-color: var(--accent-edge);
  }
  .reminder-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .progress {
    margin-top: 10px;
    width: 100%;
    height: 4px;
    background: oklch(from var(--chrome-2) calc(l + 0.04) c h);
    border-radius: 2px;
    overflow: hidden;
  }
  .progress-bar {
    height: 100%;
    width: 30%;
    background: var(--accent);
    border-radius: 2px;
    animation: indeterminate 1.4s ease-in-out infinite;
  }
  .progress-hint {
    font-size: 11px;
    margin-top: 6px;
  }
  @keyframes indeterminate {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(330%);
    }
  }
  /* Plain caption, not a section header — uppercase + tracking made this
     status line outrank the real <h3>s above it. */
  .git-fresh {
    margin-bottom: 4px;
    font-size: 11px;
  }
  .trash-list {
    list-style: none;
    margin: 0 0 8px;
    padding: 0;
    max-height: 180px;
    overflow-y: auto;
  }
  .trash-list li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 5px 0;
    border-bottom: 1px solid var(--chrome-edge);
  }
  .trash-list li:last-child {
    border-bottom: 0;
  }
  .git-age {
    color: var(--ink-3);
    font-family: "JetBrains Mono", monospace;
  }
  textarea {
    width: 100%;
    padding: 8px 10px;
    background: oklch(from var(--chrome-2) calc(l + 0.03) c h);
    border: 1px solid var(--chrome-edge);
    color: var(--ink);
    border-radius: 5px;
    font-family: "JetBrains Mono", monospace;
    font-size: 11.5px;
    line-height: 1.45;
    outline: none;
    resize: vertical;
  }
  textarea:focus {
    border-color: var(--accent-edge);
  }
  .key {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 14px;
    align-items: center;
    margin-bottom: 12px;
  }
  .key:last-of-type {
    margin-bottom: 0;
  }
  .key > .key-info {
    cursor: default;
  }
  .key .k {
    display: block;
    font-size: 13px;
    color: var(--ink);
    margin-bottom: 2px;
  }
  .key .v {
    display: block;
    font-size: 11px;
    color: var(--ink-3);
  }
  .key-input {
    display: flex;
    gap: 6px;
  }
  .key-input input {
    flex: 1;
    padding: 6px 10px;
    background: oklch(from var(--chrome-2) calc(l + 0.03) c h);
    border: 1px solid var(--chrome-edge);
    color: var(--ink);
    border-radius: 5px;
    font-family: "JetBrains Mono", monospace;
    font-size: 12px;
    outline: none;
  }
  .key-input input:focus {
    border-color: var(--accent-edge);
  }
  .key-input input:disabled {
    color: var(--ink-3);
    background: var(--chrome);
  }
  button {
    padding: 6px 10px;
    background: oklch(from var(--chrome-2) calc(l + 0.04) c h);
    color: var(--ink-2);
    border: 1px solid var(--chrome-edge);
    border-radius: 5px;
    font: inherit;
    font-size: 11.5px;
    cursor: pointer;
  }
  button:hover {
    color: var(--ink);
    background: oklch(from var(--chrome-2) calc(l + 0.07) c h);
  }
  button.primary {
    background: var(--accent-soft);
    color: var(--accent);
    border-color: var(--accent-edge);
  }
  button.primary:hover {
    background: oklch(from var(--accent) l c h / 0.28);
  }
  button.primary:disabled {
    opacity: 0.55;
    cursor: progress;
  }
  button.danger {
    color: oklch(0.7 0.15 25);
    border-color: oklch(0.7 0.15 25 / 0.4);
  }
  button.danger:hover {
    background: oklch(0.7 0.15 25 / 0.12);
  }
  .muted {
    color: var(--ink-3);
    font-size: 11.5px;
    line-height: 1.5;
    margin: 0 0 10px;
  }
  .error {
    margin-top: 10px;
    color: oklch(0.65 0.18 25);
    font-size: 11.5px;
  }
</style>
