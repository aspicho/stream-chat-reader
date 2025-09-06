<script lang="ts">
    import { message_queue } from "$lib/shared.svelte";
    import Message from "./message.svelte";


</script>


<div class="approve-queue">
    <h2>Approve Queue</h2>
    
    <div class="list">
        {#each message_queue as message (message.id)}
            <div class="list-item">

                <Message {message} />

            </div>
        {/each}
    </div>
</div>

