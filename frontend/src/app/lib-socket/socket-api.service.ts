import { Injectable } from '@angular/core';

@Injectable({
    providedIn: 'root'
})
export class SocketApiService {

    public socket: WebSocket | null = null

    // public onChange: (val: string) => void = () => { };
    public handlerByOpening: () => void = () => { };
    public handlerByClosing: () => void = () => { };
    public handlerByMessages: (val: string) => void = () => { };

    constructor() {
        console.log(`SocketApiService()`); // #
    }
    // 'ws'
    public connect(pathName: string): void {
        this.disconnect();

        const { location } = window
        const proto = location.protocol.startsWith('https') ? 'wss' : 'ws';
        const wsUri = `${proto}://${location.host}/${pathName}`;

        console.log('Connecting...'); // #
        this.socket = new WebSocket(wsUri);

        this.socket.onopen = () => {
            console.log('Connected'); // #
            if (!!this.handlerByOpening) {
                this.handlerByOpening();
            }
        }

        this.socket.onmessage = (ev: MessageEvent<any>) => {
            console.log('Received:' + ev.data, 'message'); // #
            if (!!this.handlerByMessages) {
                this.handlerByMessages(ev.data);
            }
        }

        this.socket.onclose = () => {
            console.log('Disconnected'); // #
            this.socket = null;
            if (!!this.handlerByClosing) {
                this.handlerByClosing();
            }
        }
    }

    public disconnect(): void {
        if (this.socket) {
            console.log('Disconnecting...'); // #
            this.socket.close();
            this.socket = null;
        }
    }

    public hasConnect(): boolean {
        return !!this.socket;
    }

    public send(text: string): void {
        if (!!this.socket) {
            this.socket.send(text);
        }
    }
}
