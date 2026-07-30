<script lang="ts">
  import type { ToolCallDelta, ToolResult } from '$lib/types'

  let { toolCall, result }: { toolCall: ToolCallDelta; result?: ToolResult } = $props()
  let expanded = $state(false)
</script>

<div class="tool-call-card" class:done={!!result}>
  <button class="tool-header" onclick={() => expanded = !expanded}>
    <span class="tool-name">{toolCall.function?.name ?? 'tool'}</span>
    <span class="tool-status">
      {#if result}
        <span class="check">&check;</span>
      {:else}
        <span class="spinner"></span>
      {/if}
    </span>
  </button>
  {#if expanded}
    <div class="tool-detail">
      <code class="tool-args">{toolCall.function?.arguments ?? '{}'}</code>
      {#if result}
        <div class="tool-result" class:error={result.output.startsWith('Error:')}>
          <pre>{result.output}</pre>
        </div>
      {:else}
        <div class="tool-running">Running...</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tool-call-card {
    border: 1px solid #2a313a;
    border-radius: 8px;
    overflow: hidden;
    margin: 4px 0;
  }
  .tool-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 6px 10px;
    background: #0a0e14;
    border: none;
    color: #e2e8f0;
    font-size: 12px;
    cursor: pointer;
  }
  .tool-header:hover { background: #111418; }
  .tool-name { color: #5eeaaf; font-weight: 500; }
  .tool-status { display: flex; align-items: center; gap: 4px; }
  .check { color: #5eeaaf; }
  .spinner {
    width: 12px; height: 12px;
    border: 2px solid #2a313a;
    border-top-color: #5eeaaf;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    display: inline-block;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  .tool-detail { padding: 8px 10px; background: #080b10; }
  .tool-args {
    display: block;
    font-size: 11px;
    color: #8892a8;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .tool-result {
    margin-top: 6px;
    padding: 6px 8px;
    background: #0d1117;
    border-radius: 4px;
    font-size: 12px;
  }
  .tool-result pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
    color: #c8d0dc;
  }
  .tool-result.error pre { color: #f87171; }
  .tool-running { color: #8892a8; font-size: 12px; font-style: italic; margin-top: 4px; }
</style>
