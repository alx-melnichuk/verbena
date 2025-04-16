import { Injectable } from '@angular/core';

let idx = 0;

@Injectable({
    providedIn: 'root'
})
export class SocketApiService {
    id = idx++;
    public socket: WebSocket | null = null

    // public onChange: (val: string) => void = () => { };
    public handlerByOpening: () => void = () => { };
    public handlerByClosing: () => void = () => { };
    public handlerByMessages: (val: string) => void = () => { };

    private uriWs: string = '';
    private isConnected: boolean = false;

    constructor() {
        console.log(`SocketApiService(id: ${this.id})`); // #
    }
    // 'ws'
    public connect(pathName: string, host?: string | null): void {
        this.disconnect();

        const { location } = window
        const proto = location.protocol.startsWith('https') ? 'wss' : 'ws';
        const host2 = !!host ? host : location.host;

        // if (host == 'localhost:4250') {
        //     host = '127.0.0.1:8080';
        // }
        this.uriWs = `${proto}://${host2}/${pathName}`;

        console.log(`Connecting with "${this.uriWs}" ...`);
        // # http://127.0.0.1:8080/ws   "ws://localhost:4250/ws"   hostSocketChat
        this.socket = new WebSocket(this.uriWs);

        this.socket.onopen = () => {
            console.log('Connected'); // #
            this.isConnected = true;
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
            this.isConnected = false;
            const socket = this.socket;
            this.socket = null;
            socket.close();
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
