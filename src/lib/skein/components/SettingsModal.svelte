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
  } from "../vault.svelte.js";
  import { exportVault } from "../vault.js";
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
    } catch (e) {
      dailyError = String(e);
    }
  }

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
  let gitBusy = $state(false);
  let gitMessage = $state<string | null>(null);
  let gitError = $state<string | null>(null);
  let gitCommitMsg = $state("");

  async function loadGitConfig() {
    try {
      const s = await getSettings();
      if (s.git_remote_url) gitRemote = s.git_remote_url;
      if (s.git_branch) gitBranch = s.git_branch;
      if (s.git_auth_kind) {
        gitAuthKind = (s.git_auth_kind as typeof gitAuthKind) || "none";
      }
    } catch (e) {
      gitError = String(e);
    }
  }

  async function refreshGitStatus() {
    try {
      gitStatusData = await gitStatus();
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
  });

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

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
  <div
    class="modal"
    role="dialog"
    aria-label="Settings"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
  >
    <header>
      <h2>Settings</h2>
      <button class="close" onclick={onClose} aria-label="Close">×</button>
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
              <button class="danger" onclick={closeVault}>close</button>
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

      {#if vaultState.vault}
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
          <div class="row">
            <input
              type="text"
              bind:value={gitCommitMsg}
              placeholder="commit message for push (optional)"
            />
          </div>

          {#if gitStatusData}
            <div class="kv">
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
</div>

<style>
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
