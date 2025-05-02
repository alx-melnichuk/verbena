import { Injectable } from '@angular/core';

import { EventWS, EWSType, EWSTypeUtil } from './socket-chat.interface';

export interface ChatConfig {
    nickname: string;
    room: number; // Stream.id
    access?: string | null | undefined; // AccessToken
}

@Injectable({
    providedIn: 'root'
})
export class SocketChatService {

    public changeDetector: () => void = () => { };
    public countOfMembers: number = 0;
    public error: string = '';

    public handlOnOpen: () => void = () => { };
    public handlOnClose: () => void = () => { };
    public handlOnError: (err: string) => void = () => { };
    public handlSend: (val: string) => void = () => { };
    public handlReceive: (val: string) => void = () => { };

    private chatConfig: ChatConfig | null = null;
    private hasJoined: boolean = false;
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

            this.initParams();
            this.uriWs = uriWs;
            console.log(`[01]SocketChatSrv.connect(${uriWs})`); // #
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

    public isJoined(): boolean {
        return this.hasConnect() && this.hasJoined;
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
        console.log(`[02]SocketChatSrv.eventOpen()`); // #
        if (!!this.socket && !!this.chatConfig) {
            this.socket.onmessage = (ev: MessageEvent<any>) => this.eventReceive(ev.data);

            this.handlOnOpen();
            // The username is taken from the authorization.
            // // Set the chat username.
            // this.sendData(EWSTypeUtil.getNameEWS(this.chatConfig.nickname));
            // Join the chat room.
            this.sendData(EWSTypeUtil.getJoinEWS(
                this.chatConfig.room,
                undefined,
                undefined,
                this.chatConfig.access || undefined,
            ));
        }
    };
    /** Processing the "close" event of the Socket. */
    private eventClose = (): void => {
        this.handlOnClose();
    };
    /** Processing the "error" event of the Socket. */
    private eventError = (err: any): void => {
        this.error = '' + err;
        console.log(`[03]SocketChatSrv.eventError(${err});`); // #
        this.handlOnError(this.error);
    };
    /** Processing the "send data" event of the Socket. */
    private eventSend = (val: string): void => {
        console.log(`[04]SocketChatSrv.eventSend(${val});`); // #
        this.handlSend(val);
    }
    /** Processing the "receive data" event of the Socket. */
    private eventReceive = (val: string): void => {
        console.log(`[05]SocketChatSrv.eventReceive(${val})`); // #
        const eventWS: EventWS | null = EventWS.parse(val);
        this.eventAnalysis(eventWS, this.chatConfig);
        this.handlReceive(val);
    };

    private initParams(): void {
        this.countOfMembers = 0;
        this.error = '';
        this.hasJoined = false;
    }

    private eventAnalysis(eventWS: EventWS | null, chatConfig: ChatConfig | null) {
        if (!eventWS || !chatConfig) {
            return;
        }
        if (eventWS.et == EWSType.Err) {
            const err: string = eventWS.getStr('err') || '';
            this.error = err;
        } else if (eventWS.et == EWSType.Count || eventWS.et == EWSType.Join || eventWS.et == EWSType.Leave) {
            if (eventWS.et == EWSType.Join && !this.hasJoined) {
                const room = eventWS.getInt('join') || -1;
                const member = eventWS.getStr('member') || '';
                this.hasJoined = (room == chatConfig.room && member == chatConfig.nickname);
            }
            const count: number = parseInt((eventWS.getStr('count') || '-1'), 10) || -1;
            this.countOfMembers = count > -1 ? count : this.countOfMembers;
        }
    }

}
