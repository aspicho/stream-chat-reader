export interface Message {
    id: string;
    platform: string;
    channel: string;
    username: string;
    content: string;
    additional_info: string | null;
    timestamp: number;
    published: boolean;
}

export const message_queue: Array<Message> = $state([]);
const message_queue_ids: Set<string> = $derived(new Set(message_queue.map(msg => msg.id)));

export const published_messages: Array<Message> = $state([]);
const published_message_ids: Set<string> = $derived(new Set(published_messages.map(msg => msg.id)));

export let websocket: WebSocket | null = null;

export function openWebsocket() {
    if (websocket) {
        return;
    }

    const host = window.location.host;
    websocket = new WebSocket(`ws://${host}/api/admin/ws`);
    
    websocket.onopen = () => {
        console.log("WebSocket connection opened");
    };

    websocket.onmessage = handleMessage;

    websocket.onclose = () => {
        console.log("WebSocket connection closed");
        websocket = null;
    };

    websocket.onerror = (error) => {
        console.error("WebSocket error:", error);
        websocket = null;
    }
}

export async function getChannels() {
    const response = await fetch('/api/channels');
    if (!response.ok) {
        throw new Error(`Failed to fetch channels: ${response.statusText}`);
    }
    const data = await response.json();
    return data.channels;
}

export async function publishMessage(id: string) {
    const response = await fetch(`/api/publish/${id}`, {
        method: 'POST'
    });
    if (!response.ok) {
        throw new Error(`Failed to publish message: ${response.statusText}`);
    }
    published_messages.unshift(message_queue.find(msg => msg.id === id)!);
    message_queue.splice(message_queue.indexOf(message_queue.find(msg => msg.id === id)!), 1);
    const data = await response.json();
    return data;
}

export async function getMessages(limit: number = 100) {
    const response = await fetch(`/api/messages?limit=${limit}`);
    if (!response.ok) {
        throw new Error(`Failed to fetch messages: ${response.statusText}`);
    }
    const data = await response.json();
    data.messages.forEach((msg: Message) => {
        if (msg.published) {
            if (!published_message_ids.has(msg.id)) {
                published_messages.push(msg);
            }
        } else {
            if (!message_queue_ids.has(msg.id)) {
                message_queue.push(msg);
            }
        }
    });
    return data.messages;
}

function decimalToHex(decimal: number): string {
    return `#${decimal.toString(16).padStart(6, '0')}`;
}

export function getUserColor(message: any): string {
    if (message.platform === 'twitch' && message.additional_info) {
        try {
            const info = JSON.parse(message.additional_info);
            if (info.display_color) {
                return decimalToHex(info.display_color);
            }
        } catch (e) {
            console.error('Failed to parse additional_info:', e);
        }
    }
    return message.platform === 'twitch' ? 'var(--twitch-color)' : 'var(--youtube-color)';
}

export function getTwitchInfo(message: any): any | null {
    if (message.platform === 'twitch' && message.additional_info) {
        try {
            return JSON.parse(message.additional_info);
        } catch (e) {
            console.error('Failed to parse additional_info:', e);
            return null;
        }
    }
    return null;
}

function handleMessage(event: MessageEvent) {
    console.log("Received message:", event.data);
    const message: Message = JSON.parse(event.data);

    if (message.published) {
        if (!published_message_ids.has(message.id)) {
            published_messages.push(message);
        }

        if (message_queue_ids.has(message.id)) {
            const index = message_queue.findIndex(msg => msg.id === message.id);
            if (index !== -1) {
                message_queue.splice(index, 1);
            }
        }
    
    } else {
        if (!message_queue_ids.has(message.id)) {
            message_queue.push(message);
        }
    }
}