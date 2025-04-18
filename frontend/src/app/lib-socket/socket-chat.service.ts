import { Injectable } from '@angular/core';
import { Observable, Subject, Subscription } from 'rxjs';

import { SocketApiService } from './socket-api.service';
import { EventWS, EWSType } from './socket-chat.interface';

export interface ChatConfig {
    nickname: string;
    room: string; // Stream.id
    access?: string | null | undefined; // AccessToken
}

const TIMEOUT_CONNECT = 60; // sec
const DEF_RESOLVE = () => { };
const DEF_REJECT = (reason: unknown) => { };

@Injectable({
    providedIn: 'root'
})
export class SocketChatService {

    public isReady: boolean = false;

    public get isConfigurat(): boolean { return !!this.chatConfig; }
    private chatConfig: ChatConfig | null = null;

    public get countOfMembers(): number { return this.innCountOfMembers; }
    private innCountOfMembers: number = 0;
    private innCountOfMembersSub: Subject<number | null> = new Subject();
    public countOfMembers$ = this.innCountOfMembersSub.asObservable();

    private innConnectResolve: () => void = DEF_RESOLVE;
    private innConnectReject: (reason: unknown) => void = DEF_REJECT;
    private innConnectTimer: number | undefined;

    private innSendMsgResolve: () => void = DEF_RESOLVE;
    private innSendMsgReject: (reason: unknown) => void = DEF_REJECT;
    private innSendMsgTimer: number | undefined;

    constructor(private socketApiService: SocketApiService) {
        console.log(`SocketChatSrv()`); // #
        this.socketApiService.handlOnClose = this.handlOnClose;
        this.socketApiService.handlOnError = this.handlOnError;
        this.socketApiService.handlOnOpen = this.handlOnOpen;
        this.socketApiService.handlOnMessage = this.handlOnMessage;
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
            // Connect to the server's web socket.
            this.socketApiService.connect(uriWs);
        }
    }
    public connect2(pathName: string, host?: string | null): Promise<void> {
        if (!this.chatConfig) {
            return Promise.reject();
        }
        this.isReady = false;
        const { location } = window;
        const proto = location.protocol.startsWith('https') ? 'wss' : 'ws';
        const host2 = host || location.host;
        const uriWs = `${proto}://${host2}/${pathName}`;
        console.log(`SocketChatSrv_.connect2() socketApiService.connect(uriWs);`); // #
        // Connect to the server's web socket.
        this.socketApiService.connect(uriWs);

        return new Promise<void>((resolve: () => void, reject: (reason: unknown) => void) => {
            console.log(`SocketChatSrv_.connect2() new Promise<void>() !!this.innConnectTimer: ${!!this.innConnectTimer}`); // #
            this.innConnectResolve = resolve;
            this.innConnectReject = reject;
            if (!!this.innConnectTimer) {
                window.clearTimeout(this.innConnectTimer);
            }
            this.innConnectTimer = window.setTimeout(() => {
                this.innConnectTimer = undefined;
                console.log(`SocketChatSrv_.connect2() setTimeout() innConnectReject();`)
                this.innConnectReject(false);
                this.clearConnectionParams();
            }, TIMEOUT_CONNECT * 1000);
        });
    }
    public sendMsg(msg: string): Promise<void> {
        this.socketApiService.send(msg);
        return new Promise<void>((resolve: () => void, reject: (reason: unknown) => void) => {
            console.log(`SocketChatSrv_.sendMsg() new Promise<void>() !!this.innSendMsgTimer: ${!!this.innSendMsgTimer}`); // #
            this.innSendMsgResolve = resolve;
            this.innSendMsgReject = reject;
            if (!!this.innSendMsgTimer) {
                window.clearTimeout(this.innSendMsgTimer);
            }
            this.innSendMsgTimer = window.setTimeout(() => {
                this.innSendMsgTimer = undefined;
                console.log(`SocketChatSrv_.sendMsg() setTimeout() innSendMsgReject();`)
                this.innSendMsgReject(false);
                this.clearSendMsgParams();
            }, TIMEOUT_CONNECT * 1000);
        });
    }
    /*
    return new Promise<void>((resolve: () => void, reject: (reason: unknown) => void) => {
        this.translate.use(locale).pipe(first())
            .subscribe({
                next: () => {
                    this.currLocale = locale;
                    this.dateAdapter.setLocale(locale);
                    HttpErrorUtil.setTranslate(this.translate);
                    resolve();
                },
                error: (err) => reject(err)
            });
    });
    */

    /** Disconnect from the server web socket chat. */
    public disconnect(): void {
        this.isReady = false;
        this.socketApiService.disconnect();
    }

    // ** Event handling in SocketApiService **

    /** Processing closing of connection. */
    public handlOnClose = (): void => {
        console.log(`SocketChatSrv.handlOnClose()`); // #
    };
    /** Processing error data. */
    public handlOnError = (err: any): void => {
        console.log(`SocketChatSrv.handlOnError() err:`, err); // #
        console.log(`SocketChatSrv_.handlOnError() this.innSendMsgReject('error');`); // #
        this.innSendMsgReject('error');
    };
    /** Processing opening of connection. */
    public handlOnOpen = (): void => {
        console.log(`SocketChatSrv.handlOnOpen()`); // #
        if (!!this.chatConfig) {
            // Set the chat user name.
            this.socketApiService.send(`{ "name": "${this.chatConfig.nickname}" }`);
            // Join the chat room (indicate your nickname, authorization token).
            // { "join": "romm_21", "access"?: AccessToken }
            const accessStr = !!this.chatConfig.access ? `,"access":"${this.chatConfig.access}"` : ``;
            this.socketApiService.send(`{ "join": "${this.chatConfig.room}"${accessStr} }`);
        }
    };
    /** Processing receiving of message. */
    public handlOnMessage = (val: string): void => {
        console.log(`SocketChatSrv.handlOnMessage() val:`, val); // #
        console.log(`SocketChatSrv_.handlOnMessage() this.innSendMsgResolve();`); // #
        this.innSendMsgResolve();
        const eventWS: EventWS | null = EventWS.parse(val);
        this.eventAnalysis(eventWS, this.chatConfig);
    };

    // ** **

    // ** Private API **

    private setCountOfMembers(value: number): void {
        if (value >= 0 && value != this.innCountOfMembers) {
            this.innCountOfMembers = value;
            this.innCountOfMembersSub.next(this.countOfMembers);
        }
    }

    private eventAnalysis(eventWS: EventWS | null, chatConfig: ChatConfig | null) {
        if (!eventWS || !chatConfig) {
            return;
        }
        if (eventWS.et == EWSType.Join) {
            // Join the client to the chat room. { join: String, member: String, count: usize }
            if (!this.isReady && eventWS.get('member') == chatConfig.nickname) {
                this.isReady = true;
                console.log(`SocketChatSrv.analysis() ev.et: Join, isReady: ${this.isReady}`)
                console.log(`SocketChatSrv_.analysis() this.innConnectResolve();`)
                this.innConnectResolve();
                this.clearConnectionParams();
            }
            this.setCountOfMembers(parseInt(eventWS.get('count') || '-1', 10) || -1);
        } else if (eventWS.et == EWSType.Leave) {
            this.setCountOfMembers(parseInt(eventWS.get('count') || '-1', 10) || -1);
        }
    }


    private clearConnectionParams(): void {
        this.innConnectResolve = DEF_RESOLVE;
        this.innConnectReject = DEF_REJECT;
        console.log(`SocketChatSrv_.clearConnectionParams() !!this.innConnectTimer: ${!!this.innConnectTimer}`); // #
        if (!!this.innConnectTimer) {
            console.log(`SocketChatSrv_.clearConnectionParams() window.clearTimeout(this.innConnectTimer);`); // #
            window.clearTimeout(this.innConnectTimer);
            this.innConnectTimer = undefined;
        }
    }

    private clearSendMsgParams(): void {
        this.innSendMsgResolve = DEF_RESOLVE;
        this.innSendMsgReject = DEF_REJECT;
        console.log(`SocketChatSrv_.clearSendMsgParams() !!this.innSendMsgTimer: ${!!this.innSendMsgTimer}`); // #
        if (!!this.innSendMsgTimer) {
            console.log(`SocketChatSrv_.clearSendMsgParams() window.clearTimeout(this.innSendMsgTimer);`); // #
            window.clearTimeout(this.innSendMsgTimer);
            this.innSendMsgTimer = undefined;
        }
    }
}
