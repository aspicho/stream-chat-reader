<script lang="ts">
    import ApproveQueue from "$lib/components/approve-queue.svelte";
    import MessageDisplay from "$lib/components/message-display.svelte";
    import MessageList from "$lib/components/message-list.svelte";
    import Settings from "$lib/components/settings.svelte";

    import { onMount } from "svelte";
    import { open_websocket } from "$lib/shared.svelte";


    onMount(() => {
        document.title = "Chat Listener Admin Page";
        open_websocket();
    });

</script>

<h1>Chat Listener Admin Page</h1>

<div>
    <div>
        <MessageDisplay />
        <MessageList />
    </div>
    <div>
        <Settings />
        <ApproveQueue />
    </div>
</div>