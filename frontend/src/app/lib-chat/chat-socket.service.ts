import { inject, Injectable } from '@angular/core';

import { SocketService } from '../lib-socket/socket.service';
import { EventWS, EWSType, EWSTypeUtil } from '../lib-socket/socket-chat.interface';

import { ChatMessageDto, ChatMessageDtoUtil } from './chat-message-api.interface';


export interface ChatConfig {
    nickname: string;
    room: number; // Stream.id
    access?: string | null | undefined; // AccessToken
}

@Injectable({
    providedIn: 'root'
})
export class ChatSocketService {

    public countOfMembers: number = 0;
    public error: string = '';

    public handlOnOpen: () => void = () => { };
    public handlOnClose: () => void = () => { };
    public handlOnError: (err: string) => void = () => { };
    public handlSend: (val: string) => void = () => { };
    public handlReceive: (val: string) => void = () => { };

    private chatConfig: ChatConfig | null = null;
    private hasJoined: boolean = false;
    private hasBlocked: boolean = false;
    private socketService = inject(SocketService);

    private oldHandlOnOpen: () => void = () => { };
    private oldHandlOnClose: () => void = () => { };
    private oldHandlOnError: (err: string) => void = () => { };
    private oldHandlSend: (val: string) => void = () => { };
    private oldHandlReceive: (val: string) => void = () => { };

    constructor() {
        console.log(`ChatSocketService()`); // #
    }

    // ** Public API **

    public config(chatConfig: ChatConfig): void {
        if (!!chatConfig.nickname && !!chatConfig.room) {
            this.chatConfig = { ...chatConfig };
        }
    }

    /** Connect to the server web socket chat. */
    public connect(pathName: string, host?: string | null): void {
        if (!!this.socketService) {
            this.oldHandlOnOpen = this.socketService.handlOnOpen;
            this.oldHandlOnClose = this.socketService.handlOnClose;
            this.oldHandlOnError = this.socketService.handlOnError;
            this.oldHandlSend = this.socketService.handlSend;
            this.oldHandlReceive = this.socketService.handlReceive;

            this.socketService.handlOnOpen = this.innHandlOnOpen;
            this.socketService.handlOnClose = this.innHandlOnClose;
            this.socketService.handlOnError = this.innHandlOnError;
            this.socketService.handlSend = this.innHandlSend;
            this.socketService.handlReceive = this.innHandlReceive;

            this.socketService.connect(pathName, host)
        }
    }
    /** Disconnect from the server's web socket. */
    public disconnect(): void {
        if (!!this.socketService) {
            this.socketService.disconnect();

            this.socketService.handlOnOpen = this.oldHandlOnOpen;
            this.socketService.handlOnClose = this.oldHandlOnClose;
            this.socketService.handlOnError = this.oldHandlOnError;
            this.socketService.handlSend = this.oldHandlSend;
            this.socketService.handlReceive = this.oldHandlReceive;
        }

    }
    public hasConnect(): boolean {
        return this.socketService?.hasConnect();
    }

    public sendData(val: string): void {
        this.socketService?.sendData(val);
    }

    public clearError(): void {
        this.socketService?.clearError()
    }

    public isJoined(): boolean {
        return this.hasConnect() && this.hasJoined;
    }

    public isBlocked(): boolean {
        return this.hasConnect() && this.hasBlocked;
    }

    // ** Private API **

    /** Processing the "open" event of the Socket. */
    private innHandlOnOpen = (): void => {
        console.log(`12_ChatSocketService.innHandlOnOpen()`); // #
        if (!!this.chatConfig) {
            // The username is taken from the authorization.
            // // Set the chat username.
            // this.sendData(EWSTypeUtil.getNameEWS(this.chatConfig.nickname));
            // Join the chat room.
            this.sendData(EWSTypeUtil.getJoinEWS(
                this.chatConfig.room,
                undefined,
                undefined,
                this.chatConfig?.access || undefined,
            ));
        }
        !!this.handlOnOpen && this.handlOnOpen();
    }
    /** Processing the "close" event of the Socket. */
    private innHandlOnClose = () => {
        console.log(`13_ChatSocketService.innHandlOnClose()`); // #
        !!this.handlOnClose && this.handlOnClose();
    };
    /** Processing the "error" event of the Socket. */
    private innHandlOnError = (err: string) => {
        console.log(`14_ChatSocketService.innHandlOnError()`); // #
        !!this.handlOnError && this.handlOnError(err);
    };
    /** Processing the "send data" event of the Socket. */
    private innHandlSend = (val: string) => {
        console.log(`15_ChatSocketService.innHandlSend(${val})`); // #
        !!this.handlSend && this.handlSend(val);
    };
    /** Processing the "receive data" event of the Socket. */
    private innHandlReceive = (val: string) => {
        console.log(`16_ChatSocketService.innHandlReceive(${val})`); // #
        this.eventAnalysis(EventWS.parse(val), this.chatConfig);
        !!this.handlReceive && this.handlReceive(val);
    };

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
        } else if (eventWS.et == EWSType.Block) {
            console.log(`ChatSocketService.eventAnalysis() EWSType.Block  eventWS.block==chatConfig.nickname hasBlocked = true`); // #
            this.hasBlocked = (eventWS.getStr('block') || '') == chatConfig.nickname ? true : this.hasBlocked;
        } else if (eventWS.et == EWSType.Unblock) {
            console.log(`ChatSocketService.eventAnalysis() EWSType.Unblock  eventWS.unblock==chatConfig.nickname hasBlocked = false`); // #
            this.hasBlocked = (eventWS.getStr('unblock') || '') == chatConfig.nickname ? false : this.hasBlocked;
        }
    }

}
