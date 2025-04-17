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
    /** Connect to the server's web socket. */
    public connect(uriWs: string): void {
        this.disconnect();

        this.uriWs = uriWs;
        console.log(`Connecting with "${this.uriWs}" ...`); // #
        // Create a web socket.
        this.socket = new WebSocket(this.uriWs);

        this.socket.onopen = () => {
            console.log('Connected'); // #
            this.isConnected = true;
            if (this.handlerByOpening != null) {
                this.handlerByOpening();
            }
        }

        this.socket.onmessage = (ev: MessageEvent<any>) => {
            console.log('Received:' + ev.data, 'message'); // #
            if (this.handlerByMessages != null) {
                this.handlerByMessages(ev.data);
            }
        }

        this.socket.onclose = () => {
            console.log('Disconnected'); // #
            this.socket = null;
            if (this.handlerByClosing != null) {
                this.handlerByClosing();
            }
        }
    }
    /** Disconnect from the server's web socket. */
    public disconnect(): void {
        if (this.socket) {
            console.log('Disconnecting...'); // #
            this.isConnected = false;
            this.socket.close();
            this.socket = null;
        }
    }

    public hasConnect(): boolean {
        return !!this.socket && this.isConnected;
    }

    public send(text: string): void {
        if (!!this.socket && this.isConnected) {
            this.socket.send(text);
        }
    }
}
