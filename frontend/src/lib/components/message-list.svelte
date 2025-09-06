<script lang="ts">
    import { onMount } from "svelte";
    import { published_messages, getMessages } from "$lib/shared.svelte";

    import Message from "./message.svelte";

    onMount(async () => {
        try {
            await getMessages(100);
            console.log("Component mounted message sent");
        } catch (error) {
            console.error("Failed to send component mounted message:", error);
        }
    });

</script>

<div class="message-list">
    <h2>Published Messages</h2>

    <div class="list">
        {#each published_messages as message (message.id)}
            <div class="list-item">
                <Message {message} />
            </div>
        {/each}
    </div>

</div>