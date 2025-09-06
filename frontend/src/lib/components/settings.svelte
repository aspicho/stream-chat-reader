<script lang="ts">
    import { onMount } from "svelte";
    import { getChannels, message_queue, publishMessage } from "$lib/shared.svelte";

    let channels: Array<{
        id: string;
        name: string;
        listen: boolean;
        platform: string;
    }> = $state([]);

    onMount(async () => {
        channels = await getChannels();
    });

    let newPlatform = $state("");
    let newChannel = $state("");
    let channel_to_delete = $state("");
    let auto_publish = $state(true);

    $effect(() => {
        if (auto_publish && message_queue.length > 0) {
            const last_message = message_queue[message_queue.length - 1];
            publishMessage(last_message.id).catch(error => {
                console.error("Failed to auto-publish message:", error);
            });
        }
    });

    $inspect(channels);
</script>

<div>
    <h2>Channel Settings</h2>

    <p>Auto publish</p>
    <div>
        <label>
            <input type="checkbox" bind:checked={auto_publish} />
            Enable Auto Publish
        </label>
    </div>

    <div class="info">
        <p>Toggle which channels to listen to for incoming messages.</p>
        {#each channels as channel (channel.id)}
            <div class="channel-item">
                <label>
                    <input type="checkbox" bind:checked={channel.listen}
                        onchange="{
                            async () => {
                                try {
                                    if (channel.listen) {
                                        await fetch(`/api/listen/${channel.platform}/${channel.name}`, { method: 'POST' });
                                    } else {
                                        await fetch(`/api/unlisten/${channel.platform}/${channel.name}`, { method: 'POST' });
                                    }
                                } catch (error) {
                                    console.error("Failed to update channel listen status:", error);
                                }
                            }
                        }" />
                    {channel.name} ({channel.platform})
                </label>
            </div>
        {/each}
    </div>

    <div>
        <p>Manage channels:</p>
        <div>
            <div class="channel-item">
                <select bind:value={newPlatform}>
                    <option value="">Select Platform</option>
                    <option value="twitch">Twitch</option>
                    <option value="youtube">YouTube</option>
                </select>
                <input type="text" placeholder="Channel Name" bind:value={newChannel} />
                <button
                    onclick="{
                        async () => {
                            try {
                                if (!newPlatform || !newChannel) {
                                    alert("Please select a platform and enter a channel name.");
                                    return;
                                }

                                await fetch(`/api/channels/${newPlatform}/${newChannel}`, { method: 'POST' });
                                channels = await getChannels();
                                newPlatform = '';
                                newChannel = '';
                            } catch (error) {
                                console.error("Failed to add channel:", error);
                            }
                        }
                    }"
                >
                    Add Channel
                </button>
            </div>
        </div>

        <div>
            <div class="channel-item">
                <select bind:value={channel_to_delete}>
                    <option value="">Select Channel to Delete</option>
                    {#each channels as channel (channel.id)}
                        <option value={channel.id}>{channel.name} ({channel.platform})</option>
                    {/each}
                </select>
                <button
                    onclick="{
                        async () => {
                            try {
                                await fetch(`/api/channels/${channels.find(c => c.id === channel_to_delete)?.platform}/${channels.find(c => c.id === channel_to_delete)?.name}`, { method: 'DELETE' });
                                channels = await getChannels();
                                channel_to_delete = '';
                            } catch (error) {
                                console.error("Failed to delete channel:", error);
                            }
                        }
                    }"
                >                    Delete Channel
                </button>
            </div>
        </div>

        <button
            onclick="{
                async () => {
                    try {
                        channels = await getChannels();
                    } catch (error) {
                        console.error("Failed to refresh channels:", error);
                    }
                }
            }"
        >
            Refresh Channels
        </button>
    </div>
</div>

<style>
    .info {
        margin-bottom: 1rem;
        font-size: 0.9rem;
        color: var(--text-secondary-color);
    }

    .channel-item {
        margin-bottom: 0.5rem;
    }

    select, input[type="text"] {
        margin-right: 0.5rem;
        padding: 0.3rem;
        border: 1px solid var(--border-color);
        border-radius: 4px;
    }

    button {
        padding: 0.3rem 0.6rem;
        border: none;
        border-radius: 4px;
        background-color: var(--published-color);
        color: white;
        cursor: pointer;
    }

    button:hover {
        background-color: var(--published-hover-color);
    }
</style>