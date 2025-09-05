<script lang="ts">
    import { message_queue } from "$lib/shared.svelte";
</script>


<div>
    I'll be an approve queue one day!

    <ul>
        {#each message_queue as message (message.id)}
            <li>
                <strong>{message.username}:</strong> {message.content} <em>({new Date(message.timestamp).toLocaleString()})</em>
                <span> - Channel: {message.channel} - Platform: {message.platform}</span>
                <span> - Additional Info: {message.additional_info}</span>
                <span> - Published: {message.published ? 'Yes' : 'No'}</span>
                <span> - ID: {message.id}</span>
            </li>
        {/each}
    </ul>
    
</div>