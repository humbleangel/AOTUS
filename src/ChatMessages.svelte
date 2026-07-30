<script lang="ts">
  import Interaction from './Interaction.svelte'
  import type { Interaction as InteractionType } from '$lib/types'
  import { InteractionRuntime } from '$lib/runtime.svelte'

  let { interactions, runtime }: {
    interactions: InteractionType[]
    runtime: InteractionRuntime
  } = $props()

  let container: HTMLDivElement | undefined = $state()
  let autoScroll = $state(true)

  $effect(() => {
    interactions
    if (autoScroll && container) {
      requestAnimationFrame(() => {
        container!.scrollTop = container!.scrollHeight
      })
    }
  })

  function handleScroll() {
    if (!container) return
    const threshold = container.scrollHeight - container.scrollTop - container.clientHeight
    autoScroll = threshold < 50
  }
</script>

<div class="messages" bind:this={container} onscroll={handleScroll}>
  {#each interactions as interaction (interaction.id)}
    <Interaction {interaction} />
  {/each}
  {#if interactions.length === 0}
    <div class="empty">Start a conversation</div>
  {/if}
</div>

<style>
  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .empty {
    text-align: center;
    color: #555;
    margin-top: 40px;
    font-size: 14px;
  }
</style>
