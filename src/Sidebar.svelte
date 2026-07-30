<script lang="ts">
  import type { Session } from '$lib/types'

  let { sessions, currentSessionId, onNewSession, onSelectSession, onDeleteSession, onRenameSession }: {
    sessions: Session[]
    currentSessionId: string
    onNewSession: () => void
    onSelectSession: (id: string) => void
    onDeleteSession: (id: string) => void
    onRenameSession: (id: string, name: string) => void
  } = $props()

  let search = $state('')
  let editingId = $state<string | null>(null)
  let editName = $state('')
  let confirmDeleteId = $state<string | null>(null)

  let filtered = $derived(
    sessions.filter(s => s.name.toLowerCase().includes(search.toLowerCase()))
  )

  function startRename(session: Session) {
    editingId = session.id
    editName = session.name
  }

  function commitRename() {
    if (editingId && editName.trim()) {
      onRenameSession(editingId, editName.trim())
    }
    editingId = null
  }

  function requestDelete(id: string) {
    confirmDeleteId = id
  }

  function confirmDelete() {
    if (confirmDeleteId) {
      onDeleteSession(confirmDeleteId)
      confirmDeleteId = null
    }
  }
</script>

<aside class="sidebar">
  <button class="new-btn" onclick={onNewSession}>+ New chat</button>

  <div class="search-box">
    <input
      type="text"
      placeholder="Search sessions..."
      bind:value={search}
    />
  </div>

  <nav class="session-list">
    {#each filtered as session (session.id)}
      <div
        class="session-item"
        class:active={session.id === currentSessionId}
        onclick={() => onSelectSession(session.id)}
        role="button"
        tabindex="0"
        onkeydown={(e) => e.key === 'Enter' && onSelectSession(session.id)}
      >
        {#if editingId === session.id}
          <input
            class="rename-input"
            bind:value={editName}
            onblur={commitRename}
            onkeydown={(e) => { if (e.key === 'Enter') commitRename(); if (e.key === 'Escape') editingId = null }}
            autofocus
            onclick={(e) => e.stopPropagation()}
          />
        {:else}
          <span
            class="session-name"
            ondblclick={() => startRename(session)}
          >{session.name}</span>
        {/if}
        <span
          class="delete-btn"
          onclick={(e) => { e.stopPropagation(); requestDelete(session.id) }}
          role="button"
          tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && requestDelete(session.id)}
        >&times;</span>
      </div>
    {/each}
  </nav>
</aside>

{#if confirmDeleteId}
  <div class="confirm-overlay" onclick={() => confirmDeleteId = null}>
    <div class="confirm-dialog" onclick={(e) => e.stopPropagation()}>
      <p>Delete this session?</p>
      <div class="confirm-actions">
        <button class="cancel-btn" onclick={() => confirmDeleteId = null}>Cancel</button>
        <button class="delete-confirm-btn" onclick={confirmDelete}>Delete</button>
      </div>
    </div>
  </div>
{/if}

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
    margin: 12px 12px 4px;
    padding: 8px;
    background: #1a1f26;
    border: 1px solid #2a313a;
    border-radius: 8px;
    color: #e2e8f0;
    font-size: 13px;
    cursor: pointer;
  }
  .new-btn:hover { background: #222830; }
  .search-box { padding: 4px 12px 8px; }
  .search-box input {
    width: 100%;
    padding: 6px 8px;
    background: #111418;
    border: 1px solid #2a313a;
    border-radius: 6px;
    color: #e2e8f0;
    font-size: 12px;
    outline: none;
  }
  .search-box input:focus { border-color: #5eeaaf; }
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
  .session-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; cursor: pointer; }
  .rename-input {
    flex: 1;
    padding: 2px 4px;
    background: #0d1117;
    border: 1px solid #5eeaaf;
    border-radius: 4px;
    color: #e2e8f0;
    font-size: 13px;
    outline: none;
  }
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
  .confirm-overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.5);
    display: flex; align-items: center; justify-content: center;
    z-index: 100;
  }
  .confirm-dialog {
    background: #111418;
    border: 1px solid #2a313a;
    border-radius: 12px;
    padding: 20px;
    min-width: 280px;
  }
  .confirm-dialog p { color: #e2e8f0; margin-bottom: 16px; font-size: 14px; }
  .confirm-actions { display: flex; gap: 8px; justify-content: flex-end; }
  .cancel-btn, .delete-confirm-btn {
    padding: 8px 16px;
    border-radius: 8px;
    border: none;
    font-size: 13px;
    cursor: pointer;
  }
  .cancel-btn { background: #1a1f26; color: #e2e8f0; }
  .delete-confirm-btn { background: #f87171; color: white; }
</style>
