<script lang="ts">
  let { selectedModel, onSend, onCancel, disabled, onModelChange }: {
    selectedModel: string
    onSend: (message: string) => void
    onCancel: () => void
    disabled: boolean
    onModelChange: (model: string) => void
  } = $props()

  let text = $state('')
  let textarea: HTMLTextAreaElement | undefined = $state()

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
