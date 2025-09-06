<script lang="ts">
    import { getTwitchInfo, getUserColor } from "$lib/shared.svelte";

    let { message } = $props();    

    let twitchInfo = getTwitchInfo(message);

</script>

<div class="message-meta">
<div>
    <span style="color: var({message.platform === 'twitch' ? '--twitch-color' : '--youtube-color'})">
        {message.platform.toUpperCase()}
    </span>
    <span>Channel: {message.channel}</span>
</div>

<span>ID: {message.id}</span>
</div>

<div class="message-content">
    <strong style="color: {getUserColor(message)}">
        {message.username}:
    </strong>
    <span class="content">
        {message.content}
    </span>
    <em>({new Date(message.timestamp).toLocaleString()})</em>
</div>

<div class="additional-info">
    {#if twitchInfo?.returning_chatter}
        <div>
            <span>Returning Chatter: {twitchInfo.returning_chatter}</span>
        </div>
    {/if}
    {#if twitchInfo?.sub_months}
        <div>
            <span>Sub Months: {twitchInfo.sub_months}</span>
        </div>
    {/if}
    {#if twitchInfo?.role}
        <div>
            <span>Role: {twitchInfo.role}</span>
        </div>
    {/if}
</div>

<style>
    .content {
        margin-left: 0.5rem;
        max-width: 750px;
    }
    .additional-info {
        display: flex;
        gap: 1rem;
        margin-bottom: 0.5rem;
        font-size: 0.9rem;
        color: var(--text-secondary-color);
    }
    .message-content {
        display: flex;
        align-items: start;
        flex-direction: column;
        gap: 0.5rem;
        margin: 0.5rem 0;
        padding-right: 1rem;
        display: flex;
        justify-content: space-between;
    }

    .message-meta {
        font-size: 0.85rem;
        color: var(--text-secondary-color);
        margin-bottom: 0.25rem;
    }

    em {
        font-size: 0.8rem;
        color: var(--text-secondary-color);
    }
</style>
