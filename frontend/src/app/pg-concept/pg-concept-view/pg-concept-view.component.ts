import { CommonModule, KeyValue } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, OnDestroy, OnInit, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';

import { StringDateTime } from 'src/app/common/string-date-time';
import { ChatMessageDto, BlockedUserDto, ParamQueryPastMsg, ChatMessageDtoUtil } from 'src/app/lib-chat/chat-message-api.interface';
import { ChatMessageService } from 'src/app/lib-chat/chat-message.service';
import { ChatSocketService } from 'src/app/lib-chat/chat-socket.service';
import { ConceptViewComponent } from 'src/app/lib-concept/concept-view/concept-view.component';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { ConfirmationData } from 'src/app/lib-dialog/confirmation/confirmation.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { ProfileDto, ProfileMiniDto, UserTokenResponseDto } from 'src/app/lib-profile/profile-api.interface';
import { EWSTypeUtil, EventWS, EWSType } from 'src/app/lib-socket/socket-chat.interface';
import { StreamDto, StreamState, StreamDtoUtil } from 'src/app/lib-stream/stream-api.interface';
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
    public chatIsEditable: boolean | null = null;
    public chatIsLoading: boolean | null = null; // Indicates that data is being loaded.

    public chatSocketService: ChatSocketService = inject(ChatSocketService);
    // List of blocked users.
    public blockedNamesIsLoading: boolean | null = null;

    public isLoadStream: boolean = false;
    // An indication that the stream is in a chat-available status.
    public isStreamAvailable: boolean = false;
    // An indication that this is the owner of the stream.
    public isStreamOwner: boolean = false;
    public profileDto: ProfileDto | null = null;
    public profileMiniDto: ProfileMiniDto | null = null;
    public profileTokensDto: UserTokenResponseDto | null = null;
    public streamDto: StreamDto | null = null;
    public timerActive: boolean | null | undefined;
    public timerIsShow: boolean | null | undefined;
    public timerValue: number | null | undefined;

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
        this.profileMiniDto = conceptResponse?.profileMiniDto;

        this.chatBlockedUsers = conceptResponse?.blockedNames || [];
        const nickname: string = this.profileDto?.nickname || '';

        const streamDto = conceptResponse?.streamDto || null;
        this.setStreamDto(streamDto || null, this.profileDto?.id || 0);

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

        const streamId: number = streamDto?.id || -1;
        const access = this.profileTokensDto?.accessToken;
        this.chatSocketService.config({ nickname, room: streamId, access });
        // If the stream status allows it, establish a connection to the socket. Otherwise, terminate it.
        this.updateSocketConnect(this.streamDto?.state);
    }

    ngOnInit(): void {
    }

    ngOnDestroy(): void {
        this.chatSocketService.disconnect(); // Disconnect to the server web socket chat.
    }

    // ** Public API **

    // Section: "panel stream admin"

    public doChangeState(isStreamOwner: boolean, streamId: number | undefined, newState: StreamState | null): void {
        if (!!isStreamOwner && !!streamId && !!newState) {
            this.toggleStreamState(isStreamOwner, streamId, newState);
        }
    }

    // Section: "Chat"

    public doModifyBlockUser(isStreamOwner: boolean, isPost: boolean, user_name: string): void {
        if (!!isStreamOwner && !!user_name) {
            if (this.chatSocketService.hasConnect()) {
                const blockUnblockEWS = isPost ? EWSTypeUtil.getBlockEWS(user_name) : EWSTypeUtil.getUnblockEWS(user_name);
                this.chatSocketService.sendData(blockUnblockEWS);
            } else {
                this.modifyBlockedUser(user_name, isPost)
                    .then(() =>
                        this.chatBlockedUsers = this.updateBlockedNames(this.chatBlockedUsers, isPost, user_name))
                    .catch((errHttp: HttpErrorResponse) => {
                        const errMsg = HttpErrorUtil.getMsgs(errHttp)[0];
                        this.alertService.showError(errMsg, `pg-concept-view.error_${isPost ? '' : 'un'}blocked`);
                    });
            }
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
                if (!!response) {
                    const streamDto: StreamDto = (response as StreamDto);
                    this.setStreamDto(streamDto, this.profileDto?.id || 0);
                    Promise.resolve().then(() => {
                        this.chatSocketService.sendData(EWSTypeUtil.getPrmStrEWS("streamDto", JSON.stringify(streamDto)));
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
            console.log(`echo: ${eventWS.getStr('echo') || ''}`);
        } else if (eventWS.et == EWSType.Msg) {
            this.addChatMsg(val);
        } else if (eventWS.et == EWSType.MsgRmv) {
            this.removedChatMsg(eventWS.getInt('msgRmv') || -1);
        } else if (eventWS.et == EWSType.Block || eventWS.et == EWSType.Unblock) {
            const isBlock = eventWS.et == EWSType.Block;
            const username = isBlock ? eventWS.getStr('block') : eventWS.getStr('unblock');
            if (!!username && isStreamOwner) {
                this.chatBlockedUsers = this.updateBlockedNames(this.chatBlockedUsers, isBlock, username);
            }
            const nickname = profileDto?.nickname || '';
            if (!!username && (isStreamOwner || nickname == username)) {
                const msg = this.translateService.instant(`pg-concept-view.user_${(!isBlock ? 'un' : '')}blocked`, { username });
                this.alertService.showWarning(msg, 'pg-concept-view.chat_commands');
            }
        } else if (eventWS.et == EWSType.PrmStr) {
            const prmStr = eventWS.getStr('prmStr') || '';
            const valStr = eventWS.getStr('valStr') || '';
            if (!isStreamOwner && prmStr == 'streamDto' && !!valStr && eventWS.getBool('isOwner')) {
                const obj = JSON.parse(valStr);
                const streamDto2 = StreamDtoUtil.create(obj);
                if (streamDto != null && streamDto.id == streamDto2.id) {
                    this.setStreamDto(streamDto2, profileDto?.id || 0);
                    // If the stream status allows it, establish a connection to the socket. Otherwise, terminate it.
                    this.updateSocketConnect(this.streamDto?.state);
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
    private updateBlockedNames(blockedNames: string[], isBlock: boolean, blockName: string): string[] {
        const userSet: Set<string> = new Set(blockedNames);
        if (isBlock) {
            userSet.add(blockName);
        } else {
            userSet.delete(blockName);
        }
        return Array.from(userSet);
    }
    private getPastChatMsgs(streamId: number | null, isSortDes?: boolean, maxDate?: StringDateTime, limit?: number): void {
        if (!streamId || streamId < 0) {
            return;
        }
        this.chatIsLoading = true;
        this.chatMessageService.getChatMessages(streamId, isSortDes, undefined, maxDate, limit)
            .then((response: ChatMessageDto[] | HttpErrorResponse | undefined) => {
                this.chatPastMsgs = (response as ChatMessageDto[]); // List of past chat messages.
            })
            .catch((error: HttpErrorResponse) => {
                console.error(`ChatMessageError:`, error);
            })
            .finally(() => {
                this.chatIsLoading = false;
                this.changeDetector.markForCheck();
            });
    }

    // Section: "panel blocked users"

    private modifyBlockedUser(blockedNickname: string, isPost: boolean): Promise<BlockedUserDto | HttpErrorResponse | undefined> {
        this.blockedNamesIsLoading = true;
        const buffPromise: Promise<unknown>[] = [];
        let idx = -1;
        if (isPost) {
            buffPromise.push(this.chatMessageService.postBlockedUser(blockedNickname));
        } else {
            idx = this.chatBlockedUsers.indexOf(blockedNickname);
            buffPromise.push(this.chatMessageService.deleteBlockedUser(blockedNickname));
        }
        return Promise.all(buffPromise)
            .then((responses) => responses[0] as BlockedUserDto)
            .catch((error: HttpErrorResponse) => {
                console.error(`${isPost ? 'PostBlockedUser' : 'DeleteBlockedUser'}Error:`, error);
                throw error;
            })
            .finally(() => {
                this.blockedNamesIsLoading = false;
                this.changeDetector.markForCheck();
            });
    }

    // Update parameters based on stream data.
    private setStreamDto(stream: StreamDto | null, profileId: number): void {
        this.streamDto = stream;
        const streamState = stream?.state || 'stopped';
        this.isStreamOwner = (this.streamDto?.userId || -1) == profileId;
        this.isStreamAvailable = streamState != 'stopped';
        this.chatIsEditable = ['preparing', 'started', 'paused'].indexOf(streamState) > -1;

        this.updateTimerProperties(stream);
    }

    private updateSocketConnect(streamState: StreamState | undefined): void {
        if (streamState == null) {
            return;
        }
        const isConnectionAvailable = streamState != 'stopped';
        if (isConnectionAvailable && !this.chatSocketService.hasConnect()) {
            this.chatSocketService.connect(WS_CHAT_PATHNAME, WS_CHAT_HOST); // Connect to the server web socket chat.
        } else if (!isConnectionAvailable && this.chatSocketService.hasConnect()) {
            this.chatSocketService.disconnect(); // Disconnect to the server web socket chat.
        }
    }

    private updateTimerProperties(stream: StreamDto | null): void {
        // live := NEW."state" IN ('preparing', 'started', 'paused');
        this.timerIsShow = (['preparing', 'started', 'paused', 'stopped'].indexOf(this.streamDto?.state || '') > -1);
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
