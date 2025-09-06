<script lang="ts">
    import { message_queue, published_messages, publishMessage } from "$lib/shared.svelte";
    import Message from "./message.svelte";

    let last_message = $derived(message_queue.slice(-1)[0]);
</script>

<div class="message-display">
    {#if last_message}
        <Message message={last_message} />
        <div class="controls">
            <button
                class="ignore-button"
                onclick={() => message_queue.splice(message_queue.indexOf(last_message), 1)}
            >
                Ignore
            </button>
            <button
                class="publish-button"
                onclick={
                    async () => {
                        try {
                            await publishMessage(last_message.id);
                        } catch (error) {
                            console.error("Failed to publish message:", error);
                        }
                    }
                }
            >
                Publish
            </button>
        </div>
    {:else}
        <div>No messages to display.</div>
    {/if}
</div>

<style>
    .message-display {
        border: 1px solid var(--border-color);
        border-radius: 8px;
        padding: 1rem;
        margin-bottom: 2rem;
        background-color: var(--card-background-color);
    }
    .publish-button, .ignore-button {
        margin-top: 0.5rem;
        padding: 0.5rem 1rem;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 1rem;
    }
    .publish-button {
        background-color: var(--published-color);
        color: white;
        margin-right: 1rem;
    }
    .publish-button:hover {
        background-color: var(--published-hover-color);
    }
    .ignore-button {
        background-color: var(--queue-color);
        color: white;
    }
    .ignore-button:hover {
        background-color: var(--queue-hover-color);
    }

</style>