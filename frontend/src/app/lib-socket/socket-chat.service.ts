import { Injectable } from '@angular/core';

import { EventWS, EWSType } from './socket-chat.interface';

export interface ChatConfig {
    nickname: string;
    room: string; // Stream.id
    access?: string | null | undefined; // AccessToken
}

@Injectable({
    providedIn: 'root'
})
export class SocketChatService {

    public changeDetector: () => void = () => { };
    public countOfMembers: number = 0;
    public error: string = '';

    private chatConfig: ChatConfig | null = null;
    private socket: WebSocket | null = null;
    private uriWs: string = '';

    constructor() {
        console.log(`SocketChatSrv()`); // #
    }

    // ** Public API **

    public config(chatConfig: ChatConfig): void {
        if (!!chatConfig.nickname && !!chatConfig.room) {
            this.chatConfig = { ...chatConfig };
        }
    }

    /** Connect to the server web socket chat. */
    public connect(pathName: string, host?: string | null): void {
        if (!!this.chatConfig) {
            const { location } = window
            const proto = location.protocol.startsWith('https') ? 'wss' : 'ws';
            const host2 = host || location.host;
            const uriWs = `${proto}://${host2}/${pathName}`;

            this.disconnect();

            this.error = '';
            this.uriWs = uriWs;
            this.log(`SocketChatSrv.connect(${uriWs})`); // #
            // Create a web socket.
            this.socket = new WebSocket(this.uriWs);

            this.socket.onclose = this.handlOnClose;
            this.socket.onerror = this.handlOnError;
            this.socket.onopen = this.handlOnOpen;
        }
    }
    /** Disconnect from the server's web socket. */
    public disconnect(): void {
        this.log(`SocketChatSrv.disconnect()`); // #
        if (this.socket) {
            this.socket.close();
            this.socket = null;
        }
    }

    public hasConnect(): boolean {
        return this.socket?.readyState == WebSocket.OPEN;
    }

    public send(msg: string): void {
        this.error = '';
        if (!!this.socket && this.hasConnect()) {
            this.log(`SocketChatSrv.send(${msg})`); // #
            this.socket.send(msg);
        }
    }

    // ** Private API **

    private handlOnClose = (): void => {
        this.log(`SocketChatSrv.handlOnClose()`); // #
    };

    private handlOnError = (err: any): void => {
        this.error = '' + err;
        this.log(`SocketChatSrv.handlOnError(${err}) changeDetector();`); // #
        this.changeDetector();
    };

    private handlOnOpen = (): void => {
        this.log(`SocketChatSrv.handlOnOpen()`); // #
        if (!!this.socket && !!this.chatConfig) {
            this.socket.onmessage = (ev: MessageEvent<any>) => this.handlOnMessage(ev.data);

            this.send(`{ "name": "${this.chatConfig.nickname}" }`);
            this.send(`{ "join": "${this.chatConfig.room}" }`);
        }
        this.log(`SocketChatSrv.handlOnOpen() changeDetector();`); // #
        this.changeDetector();
    };

    private handlOnMessage = (val: string): void => {
        this.log(`SocketChatSrv.handlOnMessage(${val})`); // #
        const eventWS: EventWS | null = EventWS.parse(val);
        this.eventAnalysis(eventWS, this.chatConfig);
        this.log(`SocketChatSrv.handlOnMessage(${val}) changeDetector();`); // #
        this.changeDetector();
    };

    private eventAnalysis(eventWS: EventWS | null, chatConfig: ChatConfig | null) {
        if (!eventWS || !chatConfig) {
            return;
        }
        if (eventWS.et == EWSType.Join || eventWS.et == EWSType.Leave) {
            this.log(`SocketChatSrv.eventAnalysis(Join || Leave) eventWS: ${JSON.stringify(eventWS)}`); // #
            eventWS.get('count')
            const count = parseInt((eventWS.get('count') || '-1'), 10) || -1;
            if (count > -1) {
                this.log(`SocketChatSrv.eventAnalysis(Join || Leave) countOfMembers=${count}`); // #
                this.countOfMembers = count;
            }
        }
    }

    private log(text: string): void {
        // console.log(text);
    }
}
