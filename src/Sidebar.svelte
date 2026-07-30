<script lang="ts">
  import type { Session } from '$lib/types'

  let { sessions, currentSessionId, onNewSession, onSelectSession, onDeleteSession }: {
    sessions: Session[]
    currentSessionId: string
    onNewSession: () => void
    onSelectSession: (id: string) => void
    onDeleteSession: (id: string) => void
  } = $props()
</script>

<aside class="sidebar">
  <button class="new-btn" onclick={onNewSession}>+ New chat</button>
  <nav class="session-list">
    {#each sessions as session (session.id)}
      <div
        class="session-item"
        class:active={session.id === currentSessionId}
        onclick={() => onSelectSession(session.id)}
        role="button"
        tabindex="0"
        onkeydown={(e) => e.key === 'Enter' && onSelectSession(session.id)}
      >
        <span class="session-name">{session.name}</span>
        <span
          class="delete-btn"
          onclick={(e) => { e.stopPropagation(); onDeleteSession(session.id) }}
          role="button"
          tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && onDeleteSession(session.id)}
        >&times;</span>
      </div>
    {/each}
  </nav>
</aside>

<style>
  .sidebar {
    width: 240px;
    background: #0d1117;
    border-right: 1px solid #1e2329;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }
  .new-btn {
    margin: 12px;
    padding: 8px;
    background: #1a1f26;
    border: 1px solid #2a313a;
    border-radius: 8px;
    color: #e2e8f0;
    font-size: 13px;
    cursor: pointer;
  }
  .new-btn:hover { background: #222830; }
  .session-list { flex: 1; overflow-y: auto; padding: 0 8px; }
  .session-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 8px 10px;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: #8892a8;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }
  .session-item:hover { background: #161b22; color: #e2e8f0; }
  .session-item.active { background: #1a1f26; color: #e2e8f0; }
  .session-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .delete-btn {
    background: none;
    border: none;
    color: #555;
    font-size: 16px;
    cursor: pointer;
    padding: 0 4px;
    opacity: 0;
  }
  .session-item:hover .delete-btn { opacity: 1; }
  .delete-btn:hover { color: #f87171; }
</style>
