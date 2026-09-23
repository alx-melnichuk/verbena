import { CommonModule, KeyValue } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import {
    Component, ViewEncapsulation, ChangeDetectionStrategy, OnDestroy, inject, ChangeDetectorRef, AfterContentInit
} from "@angular/core";
import { ActivatedRoute } from "@angular/router";
import { TranslateService } from "@ngx-translate/core";
import { environment as env } from "../../../environments/environment";
import { LocaleSrv } from "../../common/locale-srv";
import { LOG_PG_BROWSE } from "../../common/routes";
import { User } from "../../common/session-srv";
import { StringDateTime } from "../../common/string-date-time";
import { Sidebar } from "../../components/sidebar/sidebar";
import { Spinner } from "../../components/spinner/spinner";
import { ItemView } from "../../components/view-item-list/view-item-list";
import { IconBox } from "../../directives/icon-box";
import { ChatMessageDto, ChatMessageDtoUtil, BlockedUserDto } from "../../lib-chat/chat-message";
import { ChatMessageSrv, CHAT_MSG_LIMIT } from "../../lib-chat/chat-message-srv";
import { ChatSocketSrv } from "../../lib-chat/chat-socket-srv";
import { PanelChat } from "../../lib-chat/panel-chat/panel-chat";
import { AlertSrv } from "../../lib-dialog/alert-srv";
import { ConfirmationData } from "../../lib-dialog/confirmation/confirmation";
import { DialogSrv } from "../../lib-dialog/dialog-srv";
import { EWSTypeUtil, EventWS, EWSType } from "../../lib-socket/socket-chat";
import { StreamDto, StreamState, StreamDtoUtil } from "../../lib-stream/stream-dto";
import { StreamSrv } from "../../lib-stream/stream-srv";
import { UserShortDto } from "../../lib-user/user-dto";
import { HttpErrorUtil } from "../../utils/http-error.util";
import { PanelBrowseCard } from "../panel-browse-card/panel-browse-card";
import { PanelBrowseState } from "../panel-browse-state/panel-browse-state";

const WS_CHAT_PATHNAME: string = env.wsChatPathname || "ws";
const WS_CHAT_HOST: string | null = env.wsChatHost || null;

export const CN_MESSAGE = {
    "minLength": 0,
    "maxLength": 255,
    "minRows": 1,
    "maxRows": 4,
};

export interface ItemViewSetMsg {
    list: ItemView[];
    isAddTop: boolean; // true/false - add to the top/bottom of the list;
    isMoreTopRows: boolean | null; // true - The following rows are there. (Only isAddTop: true); null - Socket data;
}

@Component({
    selector: 'app-pg-browse-details',
    exportAs: "appPgBrowseDetails",
    standalone: true,
    imports: [CommonModule, Spinner, Sidebar, PanelBrowseState, PanelBrowseCard, PanelChat, IconBox],
    templateUrl: './pg-browse-details.html',
    styleUrl: './pg-browse-details.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [ChatSocketSrv],
})
export class PgBrowseDetails implements AfterContentInit, OnDestroy {
    private alertSrv: AlertSrv = inject(AlertSrv);
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private chatMessageSrv: ChatMessageSrv = inject(ChatMessageSrv);
    private dialogSrv: DialogSrv = inject(DialogSrv);
    private route: ActivatedRoute = inject(ActivatedRoute);
    public chatSocketSrv: ChatSocketSrv = inject(ChatSocketSrv);
    public localeSrv: LocaleSrv = inject(LocaleSrv);
    private streamSrv: StreamSrv = inject(StreamSrv);
    private translateSrv: TranslateService = inject(TranslateService);

    public accessToken: string | null = this.route.snapshot.data["accessToken"] || null;
    // List of new blocked users.
    public chatBlockedUsers: string[] = this.route.snapshot.data["browseStream"]?.blockedNames || [];
    // Indicates that the user can send messages to the chat.
    public chatIsEditable: boolean | null = null;
    // A list of message IDs that have been removed from the chat. 
    public chatDeleteIds: number[] = [];
    // Indicates that data is being loaded.
    public chatIsLoading: boolean | null | undefined;
    public chatIsReset: boolean | null | undefined;
    public chatItemSetMsg: ItemViewSetMsg | null | undefined;
    public chatCnMsg = CN_MESSAGE;
    private chatSizeRows: number = 3;
    // The maximum size of rows in the buffer.
    public chatMaxSizeRows: number | null | undefined = CHAT_MSG_LIMIT * this.chatSizeRows;
    // List of blocked users.
    public blockedNamesIsLoading: boolean | null | undefined;
    public isLoadStream: boolean = false;
    public isSidebarLfOpen: boolean = false;
    public isSidebarRgOpen: boolean = false;
    // An indication that the stream is in a chat-available status.
    public isStreamAvailable: boolean = false;
    // An indication that this is the owner of the stream.
    public isStreamOwner: boolean = false;
    // To disable the jumping effect of the "stream-video" panel at startup.
    public isStreamVideo = false;
    public streamDto: StreamDto | null = null;
    public timerActive: boolean | null | undefined;
    public timerIsShow: boolean | null | undefined;
    public timerValue: number | null | undefined;
    public user: User | null = this.route.snapshot.data["user"] || null;
    public userShortDto: UserShortDto | null = this.route.snapshot.data["browseStream"]?.userShortDto || null;

    constructor() {
        if (env.logLevel & LOG_PG_BROWSE) { console.info(`PgBrowseDetails(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }

        const streamDto = this.route.snapshot.data["browseStream"]?.streamDto || null;
        this.setStreamDto(streamDto || null, this.user?.id || 0);

        this.chatSocketSrv.handlOnError = (err: string) => {
            console.error(`SocketErr:`, err);
            this.changeDetector.markForCheck();
        };
        this.chatSocketSrv.handlReceive = (val: string) => {
            this.handlReceiveChat(val, this.isStreamOwner, this.streamDto, this.user);
            this.changeDetector.markForCheck();
        };
        this.chatSocketSrv.handlOnOpen = () => {
            this.changeDetector.markForCheck();
        };

        const nickname: string = this.user?.nickname || "";
        const streamId: number = streamDto?.id || -1;
        this.chatSocketSrv.config({ nickname, room: streamId, access: this.accessToken });
        // If the stream status allows it, establish a connection to the socket. Otherwise, terminate it.
        this.updateSocketConnect(this.streamDto?.state);

        const chatMsgs: ChatMessageDto[] = this.route.snapshot.data["chatMsgList"] || [];
        this.chatItemSetMsg = { list: chatMsgs.slice(), isAddTop: true, isMoreTopRows: false };
        this.chatIsReset = true;
    }

    ngOnDestroy(): void {
        this.chatSocketSrv.disconnect(); // Disconnect to the server web socket chat.
    }

    // To disable the jumping effect of the "stream-video" panel at startup.
    ngAfterContentInit(): void {
        this.isStreamVideo = true;
        this.changeDetector.markForCheck();
    }

    // ** Public API **

    // ** Side Left and Right **

    public clearEvent(event: Event): void {
        event.preventDefault();
        event.stopPropagation();
    }

    // Section: "panel browse card"

    public doChangeState(isStreamOwner: boolean, streamId: number | undefined, newState: StreamState | null, isLoadStream: boolean): void {
        if (!!isStreamOwner && !!streamId && !!newState && !isLoadStream) {
            this.toggleStreamState(isStreamOwner, streamId, newState);
        }
    }

    // Section: "Chat"


    public doModifyBlockUser(isStreamOwner: boolean, isBlock: boolean, user_name: string): void {
        if (!!isStreamOwner && !!user_name) {
            if (this.chatSocketSrv.hasConnect()) {
                const blockUnblockEWS = isBlock ? EWSTypeUtil.getBlockEWS(user_name) : EWSTypeUtil.getUnblockEWS(user_name);
                this.chatSocketSrv.sendData(blockUnblockEWS);
            } else {
                this.modifyBlockedUser(user_name, isBlock)
                    .then(() =>
                        this.chatBlockedUsers = this.updateBlockedNames(this.chatBlockedUsers, isBlock, user_name))
                    .catch((err: HttpErrorResponse) => {
                        const errMsg = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                        this.alertSrv.showError(errMsg, `pg-browse-details.error_${isBlock ? "" : "un"}blocked`);
                        throw err;
                    });
            }
        }
    }
    public doSendMessage(newMessage: string | null): void {
        const msgVal = (newMessage || "").trim();
        if (!!msgVal) {
            this.chatSocketSrv.sendData(EWSTypeUtil.getMsgEWS(msgVal));
        }
    }
    public doEditMessage(keyValue: KeyValue<number, string> | null): void {
        const id = keyValue?.key;
        const msgPut = (keyValue?.value || "").trim();
        if (!!id && id > 0 && !!msgPut) {
            this.chatSocketSrv.sendData(EWSTypeUtil.getMsgPutEWS(msgPut, id));
        }
    }
    public doCutMessage(keyValue: KeyValue<number, string>): void {
        if (!keyValue || !keyValue.key || !keyValue.value) {
            return;
        }
        const msg = keyValue.value.slice(0, 45) + (keyValue.value.length > 45 ? "..." : "");
        const message = this.translateSrv.instant("panel-browse-view.sure_you_want_delete_message", { message: msg });
        this.dialogSrv.openConfirmation(message, "", { btnNameCancel: "buttons.no", btnNameAccept: "buttons.yes" })
            .then((res) => {
                if (!!res) {
                    this.chatSocketSrv.sendData(EWSTypeUtil.getMsgCutEWS("", keyValue.key));
                }
            });

    }
    public doRmvMessage(chMsgId: number): void {
        if (!chMsgId || chMsgId < 0) {
            return;
        }
        const message = this.translateSrv.instant("panel-browse-view.sure_you_want_permanently_delete_message");
        this.dialogSrv.openConfirmation(message, "", { btnNameCancel: "buttons.no", btnNameAccept: "buttons.yes" })
            .then((res) => {
                if (!!res) {
                    this.chatSocketSrv.sendData(EWSTypeUtil.getMsgRmvEWS(chMsgId));
                }
            });
    }

    public doLoadSet(isAddTop: boolean, date: StringDateTime | undefined, count: number): Promise<void> {
        const streamId = this.streamDto?.id;
        if (!streamId || streamId <= 0) {
            return Promise.resolve();
        }
        const isSortDes = !isAddTop;
        const minDate = (!isAddTop ? undefined : date);
        const maxDate = (!isAddTop ? date : undefined);
        const limit = count > 0 ? count : (CHAT_MSG_LIMIT + (isAddTop ? 1 : 0));
        this.chatIsReset = false;
        if (this.chatDeleteIds.length > 0) {
            this.chatDeleteIds = [];
        }
        this.chatIsLoading = true;
        return this.chatMessageSrv.getChatMessages(streamId, isSortDes, minDate, maxDate, limit)
            .then((response: ChatMessageDto[] | HttpErrorResponse | undefined) => {
                const chatMsgs = (response as ChatMessageDto[]);
                const isMoreTopRows = chatMsgs.length == (CHAT_MSG_LIMIT + 1);
                if (isMoreTopRows) {
                    chatMsgs.pop();
                }
                // When requesting previous data, a reverse is required.
                const list = isSortDes ? chatMsgs : chatMsgs.slice().reverse();
                this.chatItemSetMsg = { list, isAddTop, isMoreTopRows };
                this.chatIsReset = date == undefined;
            })
            .catch((err: HttpErrorResponse) => {
                const errMsg = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(errMsg, "pg-browse-list.error_get_streams_by_tag");
                throw err;
            })
            .finally(() => {
                this.chatIsLoading = false;
                this.changeDetector.markForCheck();
            });
    }


    // ** Private API **

    // Section: "panel browse card"

    private toggleStreamState(isStreamOwner: boolean, streamId: number | null, streamState: StreamState | null): void {
        if (!isStreamOwner || !streamId || streamId < 0 || !streamState) {
            return;
        }
        this.isLoadStream = true;
        this.streamSrv.toggleStreamState(streamId, streamState)
            .then((response: StreamDto | HttpErrorResponse) => {
                if (!!response) {
                    const streamDto: StreamDto = (response as StreamDto);
                    this.setStreamDto(streamDto, this.user?.id || 0);
                    Promise.resolve().then(() => {
                        this.chatSocketSrv.sendData(EWSTypeUtil.getPrmStrEWS("streamDto", JSON.stringify(streamDto)));
                    });
                }
            })
            .catch((err: HttpErrorResponse) => {
                const appError = (typeof (err?.error || "") == "object" ? err.error : {});
                const title = "pg-browse-details.error_update_stream";

                if (err.status == 409 && appError["code"] == "Conflict" && appError["message"] == "exist_is_active_stream") {
                    const errParams = appError["params"] || {};
                    const link = this.streamSrv.getLinkToStream(errParams["activeStream"]["id"] || -1, false);
                    const name = errParams["activeStream"]["title"] || "";
                    const confirmData: ConfirmationData = {
                        messageHtml: this.translateSrv.instant("pg-browse-details.exist_is_active_stream", { link, name }),
                    };
                    this.dialogSrv.openConfirmation(
                        "", title, { btnNameCancel: null, btnNameAccept: "buttons.ok" }, { data: confirmData });
                } else {
                    const errMsg = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                    this.alertSrv.showError(errMsg, title);
                    throw err;
                }
            })
            .finally(() => {
                this.isLoadStream = false;
                this.changeDetector.markForCheck();
            });
    }

    // Section: "Chat"

    private handlReceiveChat = (val: string, isStreamOwner: boolean, streamDto: StreamDto | null, user: User | null): void => {
        const eventWS = EventWS.parse(val);
        if (!eventWS) {
            return;
        }
        if (eventWS.et == EWSType.Err) {
            console.error(`Socket-err:`, val);
            // const errHttp = new HttpErrorResponse({ error: { code: eventWS.getStr("code"), message: eventWS.getStr("message") } });
            const status = eventWS.getInt("status") || 500;
            const errHttp = new HttpErrorResponse({ error: { status, message: eventWS.getStr("message") } });
            const errMsg = HttpErrorUtil.mapErrMsgObjs(status, errHttp.error)?.[0].msg || "error.server_api_call";
            this.alertSrv.showError(errMsg, "pg-browse-details.error_socket");
        } else if (eventWS.et == EWSType.Echo) {
            console.info(`echo: ${eventWS.getStr("echo") || ""}`);
        } else if (eventWS.et == EWSType.Msg) {
            const obj = JSON.parse(val);
            const chatMsg = ChatMessageDtoUtil.create(obj);
            if (chatMsg.id > 0 && !!chatMsg.member && !!chatMsg.date && chatMsg.date.length == 24) {
                this.chatItemSetMsg = { list: [chatMsg], isAddTop: true, isMoreTopRows: null };  // List of new chat messages.
                this.chatIsReset = false;
            }
        } else if (eventWS.et == EWSType.MsgRmv) {
            const id = eventWS.getInt("msgRmv") || -1;
            if (id > 0) { // List of IDs of permanently deleted chat messages.
                this.chatDeleteIds = [id];
            }
        } else if (eventWS.et == EWSType.Block || eventWS.et == EWSType.Unblock) {
            const isBlock = eventWS.et == EWSType.Block;
            const username = isBlock ? eventWS.getStr("block") : eventWS.getStr("unblock");
            if (!!username && isStreamOwner) {
                this.chatBlockedUsers = this.updateBlockedNames(this.chatBlockedUsers, isBlock, username);
            }
            const nickname = user?.nickname || "";
            if (!!username && (isStreamOwner || nickname == username)) {
                const msg = this.translateSrv.instant(`pg-browse-details.user_${(!isBlock ? "un" : "")}blocked`, { username });
                this.alertSrv.showWarning(msg, "pg-browse-details.chat_commands");
            }
        } else if (eventWS.et == EWSType.PrmStr) {
            const prmStr = eventWS.getStr("prmStr") || "";
            const valStr = eventWS.getStr("valStr") || "";
            if (!isStreamOwner && prmStr == "streamDto" && !!valStr && eventWS.getBool("isOwner")) {
                const obj = JSON.parse(valStr);
                const streamDto2 = StreamDtoUtil.create(obj);
                if (streamDto != null && streamDto.id == streamDto2.id) {
                    this.setStreamDto(streamDto2, user?.id || 0);
                    // If the stream status allows it, establish a connection to the socket. Otherwise, terminate it.
                    this.updateSocketConnect(this.streamDto?.state);
                }
            }
        }
    };

    private updateBlockedNames(blockedNames: string[], isBlock: boolean, blockName: string): string[] {
        const userSet: Set<string> = new Set(blockedNames);
        if (isBlock) {
            userSet.add(blockName);
        } else {
            userSet.delete(blockName);
        }
        return Array.from(userSet);
    }

    // Section: "panel blocked users"

    private modifyBlockedUser(blockedNickname: string, isPost: boolean): Promise<BlockedUserDto | HttpErrorResponse | undefined> {
        this.blockedNamesIsLoading = true;
        const buffPromise: Promise<unknown>[] = [];
        buffPromise.push(isPost
            ? this.chatMessageSrv.postBlockedUser(blockedNickname)
            : this.chatMessageSrv.deleteBlockedUser(blockedNickname)
        );
        return Promise.all(buffPromise)
            .then((responses) => responses[0] as BlockedUserDto)
            .catch((error: HttpErrorResponse) => {
                console.error(`${isPost ? "PostBlockedUser" : "DeleteBlockedUser"}Error:`, error);
                throw error;
            })
            .finally(() => {
                this.blockedNamesIsLoading = false;
                this.changeDetector.markForCheck();
            });
    }

    // Update parameters based on stream data.
    private setStreamDto(stream: StreamDto | null, userId: number): void {
        this.streamDto = stream;
        const streamState = stream?.state || "stopped";
        this.isStreamOwner = (this.streamDto?.userId || -1) == userId;
        this.isStreamAvailable = streamState != "stopped";
        this.chatIsEditable = ["preparing", "started", "paused"].indexOf(streamState) > -1;

        this.updateTimerProperties(stream);
    }

    private updateSocketConnect(streamState: StreamState | undefined): void {
        if (streamState == null) {
            return;
        }
        const isConnectionAvailable = streamState != "stopped";
        if (isConnectionAvailable && !this.chatSocketSrv.hasConnect()) {
            this.chatSocketSrv.connect(WS_CHAT_PATHNAME, WS_CHAT_HOST); // Connect to the server web socket chat.
        } else if (!isConnectionAvailable && this.chatSocketSrv.hasConnect()) {
            this.chatSocketSrv.disconnect(); // Disconnect to the server web socket chat.
        }
    }

    private updateTimerProperties(stream: StreamDto | null): void {
        // live := NEW."state" IN ("preparing", "started", "paused");
        this.timerIsShow = (["preparing", "started", "paused", "stopped"].indexOf(this.streamDto?.state || "") > -1);
        this.timerActive = false;
        this.timerValue = 0;

        if (stream != null && this.timerIsShow) {
            this.timerActive = stream.state == StreamState.started;
            if (!this.timerActive && stream.state == StreamState.preparing) {
                this.timerActive = !!stream.starttime && stream.starttime > (new Date());
            }
            this.timerValue = this.getTimerValue(stream.starttime, stream.started, stream.paused, stream.stopped);
        }
    }
    private getTimerValue(starttime: Date | null, started: Date | null, paused: Date | null, stopped: Date | null): number {
        let result: number = 0;
        const currDate = new Date();
        if (!!starttime && currDate < starttime) {
            result = Math.floor((currDate.getTime() - starttime.getTime()) / 1000);
        }
        if (!!started && currDate >= started) {
            result = Math.floor((currDate.getTime() - started.getTime()) / 1000);
        }
        if (!!paused && !!started) {
            result = Math.floor((paused.getTime() - started.getTime()) / 1000);
        }
        if (!!stopped && !!started) {
            result = Math.floor((stopped.getTime() - started.getTime()) / 1000);
        }
        return result;
    }
}
