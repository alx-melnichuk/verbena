import { Injectable } from '@angular/core';
import { SocketApiService } from './socket-api.service';
import { EventWS, EWSType } from './socket-chat.interface';

let idx = 0;

export interface ChatConfig {
    nickname: string;
    room: string; // Stream.id
    access?: string | null | undefined; // AccessToken
}

@Injectable({
    providedIn: 'root'
})
export class SocketChatService {
    id = idx++;

    public isChatReady: boolean = false;
    public get isConfigurat(): boolean {
        return !!this.chatConfig;
    }
    public countOfMembers: number = 0;

    private chatConfig: ChatConfig | null = null;

    constructor(private socketApiService: SocketApiService) {
        const srvApiId = this.socketApiService.id;
        console.log(`SocketChatService(id: ${this.id}) srvApiId: ${srvApiId}`); // #

        this.socketApiService.handlerByOpening = this.wsHandlerByOpening;
        this.socketApiService.handlerByMessages = this.wsHandlerByMessages;
        this.socketApiService.handlerByClosing = this.wsHandlerByClosing;
    }

    // ** Public API **

    public config(chatConfig: ChatConfig): void {
        this.chatConfig = { ...chatConfig };
    }
    /** Connect to the server web socket chat. */
    public connect(pathName: string, host?: string | null): void {
        if (!!this.chatConfig) {
            const { location } = window
            const proto = location.protocol.startsWith('https') ? 'wss' : 'ws';
            const host2 = !!host ? host : location.host;
            const uriWs = `${proto}://${host2}/${pathName}`;

            this.socketApiService.connect(uriWs);
        }
    }
    /** Disconnect from the server web socket chat. */
    public disconnect(): void {
        this.isChatReady = false;
        this.socketApiService.disconnect();
    }

    public wsHandlerByOpening = (): void => {
        console.log(`PgConceptView.handlerByOpening()`); // #
        if (!!this.chatConfig) {
            // Set the chat user name.
            this.socketApiService.send(`{ "name": "${this.chatConfig.nickname}" }`);
            // Join the chat room (indicate your nickname, authorization token).
            // { "join": "romm_21", "access"?: AccessToken }
            const accessStr = !!this.chatConfig.access ? `,"access":"${this.chatConfig.access}"` : ``;
            this.socketApiService.send(`{ "join": "${this.chatConfig.room}"${accessStr} }`);
        }
    };

    public wsHandlerByClosing = (): void => {
        console.log(`PgConceptView.handlerByClosing()`); // #
    };

    public wsHandlerByMessages = (value: string): void => {
        console.log(`PgConceptView.handlerByMessages(${value})`); // #
        const eventWS: EventWS | null = EventWS.parse(value);
        this.eventAnalysis(eventWS, this.chatConfig);

    };

    // ** Private API **

    private eventAnalysis(eventWS: EventWS | null, chatConfig: ChatConfig | null) {
        if (!eventWS || !chatConfig) {
            return;
        }
        if (eventWS.et == EWSType.Join) {
            // Join the client to the chat room. { join: String, member: String, count: usize }
            if (!this.isChatReady && eventWS.get('member') == chatConfig.nickname) {
                this.isChatReady = true;
                console.log(`eventWS.et: Join, isChatReady: ${this.isChatReady}`)
            }
            const count = parseInt(eventWS.get('count') || '-1', 10) || -1;
            if (count > -1) {
                this.countOfMembers = count;
            }
            console.log(`eventWS.et: Join, count: ${count}`)
        }
    }

}
