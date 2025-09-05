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
export const published_messages: Array<Message> = $state([]);
export const settings = {};
export const socket_requests = [];
export const socket_responses: Array<any> = $state([]);

export let websocket: WebSocket | null = null;

// {"id":2124127368460064141820345342462688672,"platform":"twitch","channel":"aspicho","username":"AspiCho","content":"Hi","additional_info":"{\"display_color\":9055202,\"id\":94525193,\"returning_chatter\":false,\"role\":\"Broadcaster\",\"sub_months\":null,\"username\":\"aspicho\"}","timestamp":1757036977783,"published":false}
// {"id":2124127438771223768873296264234374022,"platform":"system","channel":"system","username":"system","content":"Message 2124127368460064141820345342462688672 published","additional_info":null,"timestamp":1757037035943,"published":true}

export function open_websocket() {
    if (websocket) {
        return;
    }

    const host = window.location.host;
    websocket = new WebSocket(`ws://${host}/api/admin/ws`);
    
    websocket.onopen = () => {
        console.log("WebSocket connection opened");
    };

    websocket.onmessage = handle_message;

    websocket.onclose = () => {
        console.log("WebSocket connection closed");
        websocket = null;
    };

    websocket.onerror = (error) => {
        console.error("WebSocket error:", error);
        websocket = null;
    }
}

function handle_message(event: MessageEvent) {
    console.log("Received message:", event.data);
    const message: Message = JSON.parse(event.data);

    if (message.published && message.platform !== "system") {
        published_messages.push(message);
    } else {
        message_queue.push(message);
    }
}