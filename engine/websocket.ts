const VALID_MESSAGE_NAMES: string[] = ["FILE_CHANGED"];

export class WebSocketMessage {
    name: string;
    data: string;
    constructor(name: string, data: string) {
        this.name = name;
        this.data = data;
    }
}

export class GameWebSocket {
    socket: WebSocket;
    messages: WebSocketMessage[];
    messages_count: number;
    open: boolean;

    constructor() {
        const host = "localhost:3000"
        const socket = new WebSocket("ws://"+host);
        socket.binaryType = "arraybuffer";

        this.socket = socket;
        this.messages = [];
        this.messages_count = 0;
        this.open = false;
    
        socket.addEventListener("open", (event) => {
            this.open = true;
        });
    
        socket.addEventListener("message", (event: MessageEvent) => {
            if (typeof event.data === "string") {
                on_text_message(this, JSON.parse(event.data))
            } else {
                on_bin_message(event.data);
            }
        });

        socket.addEventListener("close", (event) => {
            this.open = false;
        })
    }
}

function on_text_message(ws: GameWebSocket, message: any) {
    if (message.name && message.data) {
        if (!VALID_MESSAGE_NAMES.includes(message.name)) {
            console.error("Unknown message:", message);
            return;
        }

        let ws_message = new WebSocketMessage(message.name, message.data);
        ws.messages[ws.messages_count] = ws_message;
        ws.messages_count += 1;
    } else {
        console.error("Unknown message:", message);
    }
}

function on_bin_message(data: ArrayBuffer) {
}
