import { Injectable } from '@angular/core';

@Injectable({
    providedIn: 'root'
})
export class SocketApiService {

    public handlOnClose: () => void = () => { };
    public handlOnError: (ev: Event) => void = this.defOnError;
    public handlOnOpen: () => void = () => { };
    public handlOnMessage: (val: string) => void = () => { };

    private uriWs: string = '';
    private socket: WebSocket | null = null

    constructor() {
        console.log(`SocketApiSrv()`); // #
    }
    /** Connect to the server's web socket. */
    public connect(uriWs: string): void {
        this.disconnect();

        this.uriWs = uriWs;
        console.log(`SocketApiSrv.connect(${uriWs})`); // #
        // Create a web socket.
        this.socket = new WebSocket(this.uriWs);

        this.socket.onclose = this.handlOnClose;
        this.socket.onerror = this.handlOnError;
        this.socket.onopen = this.handlOnOpen;
        this.socket.onmessage = (ev: MessageEvent<any>) => this.handlOnMessage(ev.data);
    }
    /** Disconnect from the server's web socket. */
    public disconnect(): void {
        console.log(`SocketApiSrv.disconnect()`); // #
        if (this.socket) {
            this.socket.close();
            this.socket = null;
        }
    }

    public hasConnect(): boolean {
        return !!this.socket;
    }

    public send(text: string): void {
        this.socket?.send(text);
    }

    private defOnError(ev: Event): void {
        console.log("WebSocket error: ", ev);
    }
}
