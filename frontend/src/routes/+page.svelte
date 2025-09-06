<script lang="ts">
    import ApproveQueue from "$lib/components/approve-queue.svelte";
    import MessageDisplay from "$lib/components/message-display.svelte";
    import MessageList from "$lib/components/message-list.svelte";
    import Settings from "$lib/components/settings.svelte";

    import { onMount } from "svelte";
    import { openWebsocket } from "$lib/shared.svelte";


    onMount(() => {
        document.title = "Chat Listener Admin Page";
        openWebsocket();
    });

</script>


<div class="container">
    <h1>Chat Listener Admin Page</h1>
    
    <MessageDisplay />
    <Settings />
    <div class="flex-container">
        <MessageList />
        <ApproveQueue />
    </div>
</div>


<style>
    :global(body) {
        background-color: var(--background-color);
    }
    .flex-container {
        display: flex;
        flex-direction: row;
        gap: 2rem;
    }

    @media (max-width: 750px) {
        .flex-container {
            flex-direction: column;
        }
    }

    .container {
        margin: 2rem;
    }
</style>