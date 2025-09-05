<script lang="ts">
    import { message_queue } from "$lib/shared.svelte";

    let last_message = $derived(message_queue.slice(-1)[0]);
</script>

<div>
    I'll display a message to aprove!

    {#if last_message}
        <div>
            <strong>{last_message.username}:</strong> {last_message.content} <em>({new Date(last_message.timestamp).toLocaleString()})</em>
            <span> - Channel: {last_message.channel} - Platform: {last_message.platform}</span>
            <span> - Additional Info: {last_message.additional_info}</span>
            <span> - Published: {last_message.published ? 'Yes' : 'No'}</span>
            <span> - ID: {last_message.id}</span>
        </div>

        <div>
            <button onclick={() => {
                console.log(`Approving message ID: ${last_message.id}`);
            }}>Approve</button>
            <button onclick={() => {
                console.log(`Rejecting message ID: ${last_message.id}`);
            }}>Ignore</button>
        </div>
    {:else}
        <div>No messages to display.</div>
    {/if}
</div>