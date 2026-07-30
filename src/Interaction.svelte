<script lang="ts">
  import type { Interaction as InteractionType } from '$lib/types'

  let { interaction }: { interaction: InteractionType } = $props()

  let showReasoning = $state(false)
  let showToolResults = $state(false)
</script>

<div class="interaction">
  <div class="prompt">{interaction.prompt}</div>
  <div class="answer" class:streaming={interaction.status === 'streaming'}>
    {#if interaction.reasoning}
      <button class="toggle-btn" onclick={() => showReasoning = !showReasoning}>
        {showReasoning ? 'Hide' : 'Show'} reasoning
      </button>
      {#if showReasoning}
        <div class="reasoning">{interaction.reasoning}</div>
      {/if}
    {/if}
    <div class="content">{interaction.answer}</div>
    {#if interaction.toolCalls.length > 0}
      <button class="toggle-btn" onclick={() => showToolResults = !showToolResults}>
        {showToolResults ? 'Hide' : 'Show'} tools ({interaction.toolCalls.length})
      </button>
      {#if showToolResults}
        <div class="tool-calls">
          {#each interaction.toolCalls as tc}
            <div class="tool-call">
              <span class="tool-name">{tc.function?.name ?? 'tool'}</span>
              <code class="tool-args">{tc.function?.arguments ?? ''}</code>
            </div>
          {/each}
        </div>
      {/if}
    {/if}
    {#if interaction.status === 'streaming'}
      <span class="cursor"></span>
    {/if}
    {#if interaction.status === 'error'}
      <div class="error">Error</div>
    {/if}
  </div>
</div>

<style>
  .interaction {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .prompt {
    background: #111418;
    padding: 12px 16px;
    border-radius: 10px;
    color: #e2e8f0;
    font-size: 14px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .answer {
    background: #0d1117;
    padding: 12px 16px;
    border-radius: 10px;
    color: #c8d0dc;
    font-size: 14px;
    line-height: 1.6;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .answer.streaming { border-left: 2px solid #5eeaaf; }
  .reasoning {
    color: #8892a8;
    font-style: italic;
    font-size: 13px;
    padding: 8px 12px;
    margin-bottom: 8px;
    background: #0a0e14;
    border-radius: 6px;
  }
  .toggle-btn {
    background: none;
    border: 1px solid #2a313a;
    color: #7dd3fc;
    font-size: 12px;
    padding: 4px 10px;
    border-radius: 6px;
    cursor: pointer;
    margin-bottom: 6px;
  }
  .toggle-btn:hover { background: #1a1f26; }
  .tool-calls { display: flex; flex-direction: column; gap: 4px; margin-top: 8px; }
  .tool-call {
    background: #0a0e14;
    padding: 6px 10px;
    border-radius: 6px;
    font-size: 12px;
  }
  .tool-name { color: #5eeaaf; font-weight: 500; }
  .tool-args { color: #8892a8; display: block; margin-top: 2px; }
  .cursor {
    display: inline-block;
    width: 8px;
    height: 16px;
    background: #5eeaaf;
    animation: blink 1s step-end infinite;
    vertical-align: text-bottom;
    margin-left: 2px;
  }
  @keyframes blink { 50% { opacity: 0; } }
  .error { color: #f87171; font-size: 13px; margin-top: 4px; }
</style>
