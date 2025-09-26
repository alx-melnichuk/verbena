import { CommonModule, KeyValue } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, OnDestroy, OnInit, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';

import { StringDateTime } from 'src/app/common/string-date-time';
import { ChatMessageService } from 'src/app/lib-chat/chat-message.service';
import { ChatMessageDto, ChatMessageDtoUtil, ParamQueryPastMsg } from 'src/app/lib-chat/chat-message-api.interface';
import { ChatSocketService } from 'src/app/lib-chat/chat-socket.service';
import { ConceptViewComponent } from 'src/app/lib-concept/concept-view/concept-view.component';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { ConfirmationData } from 'src/app/lib-dialog/confirmation/confirmation.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { ProfileDto, UserTokenResponseDto } from 'src/app/lib-profile/profile-api.interface';
import { EventWS, EWSType, EWSTypeUtil } from 'src/app/lib-socket/socket-chat.interface';
import { StreamDto, StreamDtoUtil, StreamState } from 'src/app/lib-stream/stream-api.interface';
import { StreamService } from 'src/app/lib-stream/stream.service';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';
import { environment } from 'src/environments/environment';

const WS_CHAT_PATHNAME: string = environment.wsChatPathname || 'ws';
const WS_CHAT_HOST: string | null = environment.wsChatHost || null;

@Component({
    selector: 'app-pg-concept-view',
    standalone: true,
    imports: [CommonModule, ConceptViewComponent],
    templateUrl: './pg-concept-view.component.html',
    styleUrl: './pg-concept-view.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [ChatSocketService],
})
export class PgConceptViewComponent implements OnInit, OnDestroy {

    public chatBlockedUsers: string[] = []; // List of new blocked users.
    public chatPastMsgs: ChatMessageDto[] = []; // List of past chat messages.
    public chatNewMsgs: ChatMessageDto[] = []; // List of new chat messages.
    public chatRmvIds: number[] = []; // List of IDs of permanently deleted chat messages.
    public chatIsBlocked: boolean | null = null; // Indication that the user is blocked.
    public chatIsEditable: boolean | null = null; // Indicates that the user can send messages to the chat.
    public chatIsLoadData: boolean | null = null; // Indicates that data is being loaded.
    public chatIsOwner: boolean | null = null;  // Indicates that the user is the owner of the chat.
    public chatMaxLen: number | null = null; // 255;
    public chatMinLen: number | null = null; // 0;
    public chatMaxRows: number | null = 4;
    public chatMinRows: number | null = 1;
    public chatNickname: string | null = null;

    public chatSocketService: ChatSocketService = inject(ChatSocketService);

    public isLoadStream = false;
    // An indication that the stream is in a chat-available status.
    public isStreamAvailable = false;
    // An indication that this is the owner of the stream.
    public isStreamOwner = false;
    public profileDto: ProfileDto | null = null;
    public profileTokensDto: UserTokenResponseDto | null = null;
    // The interval for displaying the timer before starting (in minutes).
    // public showTimerBeforeStart: number | null | undefined;  // ??
    public streamDto: StreamDto | null = null;

    private alertService: AlertService = inject(AlertService);
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private chatMessageService: ChatMessageService = inject(ChatMessageService);
    private dialogService: DialogService = inject(DialogService);
    private route: ActivatedRoute = inject(ActivatedRoute);
    private streamService: StreamService = inject(StreamService);
    private translateService: TranslateService = inject(TranslateService);

    constructor() {
        this.profileDto = this.route.snapshot.data['profileDto'];
        this.profileTokensDto = this.route.snapshot.data['profileTokensDto'];
        this.chatPastMsgs = this.route.snapshot.data['chatMsgList'];
        const conceptResponse = this.route.snapshot.data['conceptResponse'];
        this.setStreamDto(conceptResponse?.streamDto || null, this.profileDto?.id || 0);
        const blockedUsers = conceptResponse?.blockedUsersDto || [];
        for (let idx = 0; idx < blockedUsers.length; idx++) {
            if (!!blockedUsers[idx].blockedNickname) {
                this.chatBlockedUsers.push(blockedUsers[idx].blockedNickname);
            }
        }
        const nickname: string = this.profileDto?.nickname || '';

        this.chatSocketService.handlOnError = (err: string) => {
            console.error(`SocketErr:`, err);
            this.changeDetector.markForCheck();
        };
        this.chatSocketService.handlReceive = (val: string) => {
            this.handlReceiveChat(val, this.isStreamOwner, this.streamDto, this.profileDto);
            this.changeDetector.markForCheck();
        };
        this.chatSocketService.handlOnOpen = () => {
            this.changeDetector.markForCheck();
        };

        const streamId: number = (!!this.streamDto ? this.streamDto.id : -1);
        const access = this.profileTokensDto?.accessToken;
        this.chatSocketService.config({ nickname, room: streamId, access });
        // Connect to the server web socket chat.
        this.chatSocketService.connect(WS_CHAT_PATHNAME, WS_CHAT_HOST);
    }

    ngOnInit(): void {
    }

    ngOnDestroy(): void {
        this.chatSocketService.disconnect();
    }

    // ** Public API **

    // Section: "panel stream admin"

    public doChangeState(isStreamOwner: boolean, streamId: number | undefined, newState: StreamState | null): void {
        if (!!isStreamOwner && !!streamId && !!newState) {
            this.toggleStreamState(isStreamOwner, streamId, newState);
        }
    }

    // Section: "Chat"

    public doBlockUser(user_name: string): void {
        if (!!user_name) {
            this.chatSocketService.sendData(EWSTypeUtil.getBlockEWS(user_name));
        }
    }
    public doUnblockUser(user_name: string): void {
        if (!!user_name) {
            this.chatSocketService.sendData(EWSTypeUtil.getUnblockEWS(user_name));
        }
    }
    public doSendMessage(newMessage: string | null): void {
        const msgVal = (newMessage || '').trim();
        if (!!msgVal) {
            this.chatSocketService.sendData(EWSTypeUtil.getMsgEWS(msgVal));
        }
    }
    public doEditMessage(keyValue: KeyValue<number, string> | null): void {
        const id = keyValue?.key;
        const msgPut = (keyValue?.value || '').trim();
        if (!!id && id > 0 && !!msgPut) {
            this.chatSocketService.sendData(EWSTypeUtil.getMsgPutEWS(msgPut, id));
        }
    }
    public doCutMessage(id: number | null): void {
        if (!!id) {
            this.chatSocketService.sendData(EWSTypeUtil.getMsgCutEWS('', id));
        }
    }
    public doRmvMessage(id: number | null): void {
        if (!!id) {
            this.chatSocketService.sendData(EWSTypeUtil.getMsgRmvEWS(id));
        }
    }
    public doQueryPastMsgs(info: ParamQueryPastMsg) {
        const streamId: number = (!!this.streamDto ? this.streamDto.id : -1);
        if (streamId > 0 && !!info && info.isSortDes != null && info.maxDate != null) {
            this.getPastChatMsgs(streamId, info.isSortDes, info.maxDate);
        }
    }

    // ** Private API **

    // Section: "panel stream admin"

    private toggleStreamState(isStreamOwner: boolean, streamId: number | null, streamState: StreamState | null): void {
        if (!isStreamOwner || !streamId || streamId < 0 || !streamState) {
            return;
        }
        this.isLoadStream = true;
        this.streamService.toggleStreamState(streamId, streamState)
            .then((response: StreamDto | HttpErrorResponse) => {
                this.setStreamDto((response as StreamDto), this.profileDto?.id || 0);
                console.log(`][ toggleStreamState() streamDto:`, this.streamDto); // #
                if (!!this.streamDto) {
                    Promise.resolve().then(() => {
                        this.chatSocketService.sendData(EWSTypeUtil.getPrmStrEWS("streamDto", JSON.stringify(this.streamDto)));
                    });
                }
            })
            .catch((error: HttpErrorResponse) => {
                const appError = (typeof (error?.error || '') == 'object' ? error.error : {});
                const title = 'pg-concept-view.error_update_stream';

                if (error.status == 409 && appError['code'] == 'Conflict' && appError['message'] == 'exist_is_active_stream') {
                    const errParams = appError['params'] || {};
                    const link = this.streamService.getLinkForVisitors(errParams['activeStream']['id'] || -1, false);
                    const name = errParams['activeStream']['title'] || '';
                    const confirmData: ConfirmationData = {
                        messageHtml: this.translateService.instant('pg-concept-view.exist_is_active_stream', { link, name }),
                    };
                    this.dialogService.openConfirmation(
                        '', title, { btnNameCancel: null, btnNameAccept: 'buttons.ok' }, { data: confirmData });
                } else {
                    console.error(`ToggleStreamStateErr:`, error);
                    this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], title);
                }
            })
            .finally(() => {
                this.isLoadStream = false;
                this.changeDetector.markForCheck();
            });
    }

    // Section: "Chat"

    // private socketConnection(nickname: string | undefined, streamId: number | undefined, state: StreamSateType | undefined) {
    //     const isStateStopped = (state || 'stopped') == 'stopped';
    //     if (!this.chatSocketService.hasConnect() && nickname != null && !!nickname && streamId != null && streamId > 0 && !isStateStopped) {
    //         // Connect to the server web socket chat.
    //         this.chatSocketService.connect(WS_CHAT_PATHNAME, WS_CHAT_HOST);
    //     }
    // }
    // private socketDisconnect() {
    //     if (this.chatSocketService.hasConnect()) {
    //         // Disconnect from the websocket chat server.
    //         this.chatSocketService.disconnect();
    //     }
    // }
    private handlReceiveChat = (val: string, isStreamOwner: boolean, streamDto: StreamDto | null, profileDto: ProfileDto | null): void => {
        const eventWS = EventWS.parse(val);
        if (!eventWS) {
            return;
        }
        if (eventWS.et == EWSType.Err) {
            console.error(`Socket-err:`, val);
            const errHttp = new HttpErrorResponse({ error: { code: eventWS.getStr('code'), message: eventWS.getStr('message') } });
            const errMsg = HttpErrorUtil.getMsgs(errHttp)[0];
            this.alertService.showError(errMsg, 'pg-concept-view.error_socket');
        } else if (eventWS.et == EWSType.Echo) {
            const echo = eventWS.getStr('echo') || '';
            console.log(`echo: ${echo}`);
        } else if (eventWS.et == EWSType.Msg) {
            this.addChatMsg(val);
        } else if (eventWS.et == EWSType.MsgRmv) {
            this.removedChatMsg(eventWS.getInt('msgRmv') || -1);
        } else if (eventWS.et == EWSType.Block || eventWS.et == EWSType.Unblock) {
            const isBlock = eventWS.et == EWSType.Block;
            const username = isBlock ? eventWS.getStr('block') : eventWS.getStr('unblock');
            if (!!username && isStreamOwner) {
                this.chatBlockedUsers = this.updateBlockedUsers(this.chatBlockedUsers, isBlock, username);
            }
            const nickname = profileDto?.nickname || '';
            if (!!username && (isStreamOwner || nickname == username)) {
                const msg = this.translateService.instant(`pg-concept-view.user_${(!isBlock ? 'un' : '')}blocked`, { username });
                this.alertService.showWarning(msg, 'pg-concept-view.chat_commands');
            }
        } else if (eventWS.et == EWSType.PrmStr) {
            const prmStr = eventWS.getStr('prmStr') || '';
            const valStr = eventWS.getStr('valStr') || '';
            console.log(`][ handlReceiveChat() prmStr: ${prmStr}, valStr: ${valStr}, isOwner: ${eventWS.getBool('isOwner')}`)
            if (!isStreamOwner && prmStr == 'streamDto' && !!valStr && eventWS.getBool('isOwner')) {
                const obj = JSON.parse(valStr);
                const streamDto2 = StreamDtoUtil.create(obj);
                console.log(`][ handlReceiveChat() this.setStreamDto(streamDto);`)
                if (streamDto != null && streamDto.id == streamDto2.id) {
                    this.setStreamDto(streamDto2, profileDto?.id || 0);
                }
            }
        }
    };
    private addChatMsg(val: string): void {
        const obj = JSON.parse(val);
        const chatMsg = ChatMessageDtoUtil.create(obj);
        if (chatMsg.id > 0 && !!chatMsg.member && !!chatMsg.date) {
            this.chatNewMsgs = [chatMsg]; // List of new chat messages.
        }
    }
    private removedChatMsg(id: number): void {
        if (id > 0) { // List of IDs of permanently deleted chat messages.
            this.chatRmvIds = [id];
        }
    }
    private updateBlockedUsers(blockedUsers: string[], isBlock: boolean, blockUser: string): string[] {
        const userSet: Set<string> = new Set(blockedUsers);
        if (isBlock) {
            userSet.add(blockUser);
        } else {
            userSet.delete(blockUser);
        }
        return Array.from(userSet);
    }
    private getPastChatMsgs(streamId: number | null, isSortDes?: boolean, maxDate?: StringDateTime, limit?: number): void {
        if (!streamId || streamId < 0) {
            return;
        }
        this.chatIsLoadData = true;
        this.chatMessageService.getChatMessages(streamId, isSortDes, undefined, maxDate, limit)
            .then((response: ChatMessageDto[] | HttpErrorResponse | undefined) => {
                this.chatPastMsgs = (response as ChatMessageDto[]); // List of past chat messages.
            })
            .catch((error: HttpErrorResponse) => {
                console.error(`ChatMessageError:`, error);
            })
            .finally(() => {
                this.chatIsLoadData = false;
                this.changeDetector.markForCheck();
            });
    }
    private setStreamDto(streamDto: StreamDto | null, profileId: number): void {
        this.streamDto = streamDto;
        this.isStreamOwner = (this.streamDto?.userId || -1) == profileId;
        this.isStreamAvailable = (this.streamDto?.state || 'stopped') != 'stopped';
    }
}
