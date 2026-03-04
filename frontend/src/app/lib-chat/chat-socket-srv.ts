import { inject, Injectable } from "@angular/core";

import { SocketSrv } from "../lib-socket/socket-srv";
import { EventWS, EWSType, EWSTypeUtil } from "../lib-socket/socket-chat";


export interface ChatConfig {
    nickname: string;
    room: number; // Stream.id
    access?: string | null | undefined; // AccessToken
}

@Injectable({
    providedIn: "root",
})
export class ChatSocketSrv {

    public countOfMembers: number = 0;
    public error: string = "";

    public handlOnOpen: () => void = () => { };
    public handlOnClose: () => void = () => { };
    public handlOnError: (err: string) => void = () => { };
    public handlSend: (val: string) => void = () => { };
    public handlReceive: (val: string) => void = () => { };

    private chatConfig: ChatConfig | null = null;
    private hasJoined: boolean = false;
    private hasOwner: boolean = false;
    private hasBlocked: boolean = false;
    private socketSrv = inject(SocketSrv);

    private oldHandlOnOpen: () => void = () => { };
    private oldHandlOnClose: () => void = () => { };
    private oldHandlOnError: (err: string) => void = () => { };
    private oldHandlSend: (val: string) => void = () => { };
    private oldHandlReceive: (val: string) => void = () => { };

    constructor() {
    }

    // ** Public API **

    public config(chatConfig: ChatConfig): void {
        if (chatConfig?.room > 0) {
            this.chatConfig = { ...chatConfig };
        }
    }

    /** Connect to the server web socket chat. */
    public connect(pathName: string, host?: string | null): void {
        if (!!this.socketSrv) {
            this.hasJoined = false;
            this.hasOwner = false;
            this.hasBlocked = false;

            this.oldHandlOnOpen = this.socketSrv.handlOnOpen;
            this.oldHandlOnClose = this.socketSrv.handlOnClose;
            this.oldHandlOnError = this.socketSrv.handlOnError;
            this.oldHandlSend = this.socketSrv.handlSend;
            this.oldHandlReceive = this.socketSrv.handlReceive;

            this.socketSrv.handlOnOpen = this.innHandlOnOpen;
            this.socketSrv.handlOnClose = this.innHandlOnClose;
            this.socketSrv.handlOnError = this.innHandlOnError;
            this.socketSrv.handlSend = this.innHandlSend;
            this.socketSrv.handlReceive = this.innHandlReceive;

            this.socketSrv.connect(pathName, host)
        }
    }
    /** Disconnect from the server"s web socket. */
    public disconnect(): void {
        if (!!this.socketSrv) {
            this.socketSrv.disconnect();

            this.hasJoined = false;
            this.hasOwner = false;
            this.hasBlocked = false;

            this.socketSrv.handlOnOpen = this.oldHandlOnOpen;
            this.socketSrv.handlOnClose = this.oldHandlOnClose;
            this.socketSrv.handlOnError = this.oldHandlOnError;
            this.socketSrv.handlSend = this.oldHandlSend;
            this.socketSrv.handlReceive = this.oldHandlReceive;
        }
    }
    public hasConnect(): boolean {
        return this.socketSrv?.hasConnect();
    }

    public sendData(val: string): void {
        if (this.hasConnect()) {
            this.socketSrv?.sendData(val);
        }
    }

    public clearError(): void {
        this.socketSrv?.clearError()
    }

    public isJoined(): boolean {
        return this.hasConnect() && this.hasJoined;
    }
    public isOwner(): boolean {
        return this.hasConnect() && this.hasOwner;
    }
    public isBlocked(): boolean {
        return this.hasConnect() && this.hasBlocked;
    }

    // ** Private API **

    /** Processing the "open" event of the Socket. */
    private innHandlOnOpen = (): void => {
        if (!!this.chatConfig) {
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
        !!this.handlOnClose && this.handlOnClose();
    };
    /** Processing the "error" event of the Socket. */
    private innHandlOnError = (err: string) => {
        !!this.handlOnError && this.handlOnError(err);
    };
    /** Processing the "send data" event of the Socket. */
    private innHandlSend = (val: string) => {
        !!this.handlSend && this.handlSend(val);
    };
    /** Processing the "receive data" event of the Socket. */
    private innHandlReceive = (val: string) => {
        this.eventAnalysis(EventWS.parse(val), this.chatConfig);
        !!this.handlReceive && this.handlReceive(val);
    };

    private eventAnalysis(eventWS: EventWS | null, chatConfig: ChatConfig | null) {
        if (!eventWS || !chatConfig) {
            return;
        }
        if (eventWS.et == EWSType.Err) {
            const err: string = eventWS.getStr("err") || "";
            this.error = err;
        } else if (eventWS.et == EWSType.Count || eventWS.et == EWSType.Join || eventWS.et == EWSType.Leave) {
            // If the user is not yet connected to the room
            if (eventWS.et == EWSType.Join && !this.hasJoined) {
                const room = eventWS.getInt("join") || -1;
                const member = eventWS.getStr("member") || "";
                this.hasJoined = (room == chatConfig.room && member == chatConfig.nickname);
                // If the user is successfully connected to the room
                if (this.hasJoined) {
                    this.hasOwner = eventWS.getBool("isOwner") || false;
                    this.hasBlocked = eventWS.getBool("isBlocked") || false;
                }
            }
            const count: number = parseInt((eventWS.getStr("count") || "-1"), 10) || -1;
            this.countOfMembers = count > -1 ? count : this.countOfMembers;
        } else if (eventWS.et == EWSType.MsgRmv) {
            const msgRmv = eventWS.getInt("msgRmv") || -1;
            if (msgRmv > 0) {

            }
        } else if (eventWS.et == EWSType.Block) {
            this.hasBlocked = (eventWS.getStr("block") || "") == chatConfig.nickname ? true : this.hasBlocked;
        } else if (eventWS.et == EWSType.Unblock) {
            this.hasBlocked = (eventWS.getStr("unblock") || "") == chatConfig.nickname ? false : this.hasBlocked;
        }
    }
}
