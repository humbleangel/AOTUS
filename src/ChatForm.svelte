<script lang="ts">
  import type { ModelInfo } from '$lib/types'

  let { models, selectedModel, onSend, onCancel, disabled, onModelChange }: {
    models: ModelInfo[]
    selectedModel: string
    onSend: (message: string) => void
    onCancel: () => void
    disabled: boolean
    onModelChange: (model: string) => void
  } = $props()

  let text = $state('')
  let textarea: HTMLTextAreaElement | undefined = $state()

  let groups = $derived.by(() => {
    const map = new Map<string, ModelInfo[]>()
    for (const m of models) {
      const list = map.get(m.provider) ?? []
      list.push(m)
      map.set(m.provider, list)
    }
    return Array.from(map.entries())
  })

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      submit()
    }
  }

  function submit() {
    const msg = text.trim()
    if (!msg) return
    onSend(msg)
    text = ''
    textarea?.focus()
  }

  $effect(() => {
    if (!disabled && textarea) textarea.focus()
  })
</script>

<form class="chat-form" onsubmit={(e) => { e.preventDefault(); submit() }}>
  <div class="toolbar">
    <select class="model-select" value={selectedModel} onchange={(e) => onModelChange(e.currentTarget.value)}>
      {#each groups as [provider, providerModels]}
        <optgroup label={provider}>
          {#each providerModels as m}
            <option value={m.name}>{m.alias || m.name}</option>
          {/each}
        </optgroup>
      {/each}
    </select>
  </div>
  <div class="input-row">
    <textarea
      bind:this={textarea}
      bind:value={text}
      onkeydown={handleKeydown}
      placeholder="Send a message…"
      rows="1"
      disabled={disabled}
    ></textarea>
    {#if disabled}
      <button type="button" class="cancel-btn" onclick={onCancel}>Stop</button>
    {:else}
      <button type="submit" class="send-btn" disabled={!text.trim()}>Send</button>
    {/if}
  </div>
</form>

<style>
  .chat-form { padding: 8px 20px 12px; border-top: 1px solid #1e2329; flex-shrink: 0; }
  .toolbar { margin-bottom: 8px; }
  .model-select {
    padding: 5px 10px;
    background: #111418;
    border: 1px solid #2a313a;
    border-radius: 6px;
    color: #e2e8f0;
    font-size: 12px;
    outline: none;
    cursor: pointer;
    max-width: 100%;
  }
  .model-select:focus { border-color: #5eeaaf; }
  .model-select optgroup { color: #8892a8; font-style: normal; font-size: 11px; }
  .model-select option { color: #e2e8f0; background: #111418; }
  .input-row { display: flex; gap: 8px; align-items: flex-end; }
  textarea {
    flex: 1;
    padding: 10px 14px;
    background: #111418;
    border: 1px solid #2a313a;
    border-radius: 10px;
    color: #e2e8f0;
    font-size: 14px;
    font-family: inherit;
    resize: none;
    outline: none;
    min-height: 42px;
    max-height: 200px;
  }
  textarea:focus { border-color: #5eeaaf; }
  textarea:disabled { opacity: 0.5; }
  .send-btn, .cancel-btn {
    padding: 10px 20px;
    border-radius: 10px;
    border: none;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
  }
  .send-btn { background: #5eeaaf; color: #090b0d; }
  .send-btn:disabled { opacity: 0.4; cursor: default; }
  .cancel-btn { background: #f87171; color: white; }
</style>
