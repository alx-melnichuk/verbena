import { Injectable } from '@angular/core';

@Injectable({
    providedIn: 'root'
})
export class SocketService {

    public error: string = '';

    public handlOnOpen: () => void = () => { };
    public handlOnClose: () => void = () => { };
    public handlOnError: (err: string) => void = () => { };
    public handlSend: (val: string) => void = () => { };
    public handlReceive: (val: string) => void = () => { };

    private socket: WebSocket | null = null;
    private uriWs: string = '';

    constructor() {
    }

    // ** Public API **

    /** Connect to the server web socket chat. */
    public connect(pathName: string, host?: string | null): void {
        this.disconnect();
        if (!!pathName) {
            const { location } = window
            const proto = location.protocol.startsWith('https') ? 'wss' : 'ws';
            this.uriWs = `${proto}://${host || location.host}/${pathName}`;
            this.error = '';
            // Create a web socket.
            this.socket = new WebSocket(this.uriWs);

            this.socket.onopen = this.eventOpen;
            this.socket.onclose = this.eventClose;
            this.socket.onerror = this.eventError;
        }
    }
    /** Disconnect from the server's web socket. */
    public disconnect(): void {
        if (this.socket) {
            this.socket.close();
            this.socket = null;
        }
    }
    public hasConnect(): boolean {
        return this.socket?.readyState == WebSocket.OPEN;
    }

    public sendData(val: string): void {
        if (!!this.socket && this.socket.readyState == WebSocket.OPEN) {
            this.eventSend(val);
            this.socket.send(val);
        }
    }

    public clearError(): void {
        this.error = '';
    }

    // ** Private API **

    /** Processing the "open" event of the Socket. */
    private eventOpen = (): void => {
        if (!!this.socket) {
            this.socket.onmessage = (ev: MessageEvent<any>) => this.eventReceive(ev.data);
            !!this.handlOnOpen && this.handlOnOpen();
        }
    };
    /** Processing the "close" event of the Socket. */
    private eventClose = (): void => {
        !!this.handlOnClose && this.handlOnClose();
    };
    /** Processing the "error" event of the Socket. */
    private eventError = (err: any): void => {
        this.error = '' + err;
        !!this.handlOnError && this.handlOnError(this.error);
    };
    /** Processing the "send data" event of the Socket. */
    private eventSend = (val: string): void => {
        !!this.handlSend && this.handlSend(val);
    }
    /** Processing the "receive data" event of the Socket. */
    private eventReceive = (val: string): void => {
        !!this.handlReceive && this.handlReceive(val);
    };
}
