import { CommonModule, KeyValue } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, OnDestroy, OnInit, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';

import { ChatMessageDto, ChatMessageDtoUtil } from 'src/app/lib-chat/chat-message-api.interface';
import { ChatMessageService } from 'src/app/lib-chat/chat-message.service';
// # import { ChatMessageDto, ChatMessageDtoUtil } from 'src/app/lib-chat/chat-message-api.interface';
import { ChatSocketService } from 'src/app/lib-chat/chat-socket.service';
import { ConceptViewComponent } from 'src/app/lib-concept/concept-view/concept-view.component';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { ConfirmationData } from 'src/app/lib-dialog/confirmation/confirmation.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { ProfileDto, ProfileTokensDto } from 'src/app/lib-profile/profile-api.interface';
import { EventWS, EWSType, EWSTypeUtil } from 'src/app/lib-socket/socket-chat.interface';
import { StreamDto, StreamState, StreamStateUtil } from 'src/app/lib-stream/stream-api.interface';
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

    public blockedUsers: string[] = [];
    // public chatAccess: string | null = null;
    public chatMaxRows: number = 4;
    public chatMinRows: number = 1;
    public chatMsgs: ChatMessageDto[] = [];
    // public chatName: string = '';
    // public chatRoom: string = '';
    public chatSocketService: ChatSocketService = inject(ChatSocketService);
    public isLoadStream = false;
    public isLoadChatMsg = false;
    public isShowTimer: boolean = false;
    // An indication that the stream is in active status. ([preparing, started, paused]) 
    public isStreamActive: boolean = false;
    // An indication that this is the owner of the stream.
    public isStreamOwner = false;
    public profileDto: ProfileDto | null = null;
    public profileTokensDto: ProfileTokensDto | null = null;
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
        // this.showTimerBeforeStart = 120; // minutes
        this.profileDto = this.route.snapshot.data['profileDto'];
        this.profileTokensDto = this.route.snapshot.data['profileTokensDto'];
        this.streamDto = this.route.snapshot.data['streamDto'];
        const chatMsgList = this.route.snapshot.data['chatMsgList'];
        this.chatMsgs = chatMsgList.concat([]);
        console.log(`PgConceptView() 1 chatMsgs:`, this.chatMsgs); // #
        // =v
        if (!!this.streamDto) { // #
            // this.streamDto.state = StreamState.waiting; // # for demo
            this.streamDto.state = StreamState.preparing; // # for demo
            // this.streamDto.state = StreamState.started; // # for demo
            // this.streamDto.state = StreamState.paused; // # for demo
            // this.streamDto.state = StreamState.stopped; // # for demo
            // this.streamDto.starttime = "2024-10-29T16:34:00.000Z";
            // this.streamDto.starttime = "2025-04-10T12:00:00.000Z";
            this.streamDto.tags = [this.streamDto.tags[0], 'tag_word02', 'tag_word03', 'tag_word04'];
            // this.streamDto.tags = [this.streamDto.tags[0], 'tag02', 'tag03', 'tag04'];
            // this.streamDto.started = "2025-04-07T14:05:00.000Z";
            // this.streamDto.stopped = "2025-04-07T16:40:00.000Z";
            this.streamDto.title = this.streamDto.title + ' Invalid stream state transition from "waiting" to "stopped".'
                + ' Invalid stream state transition from "waiting" to "stopped".';
        }
        // # this.chatMsgs = this.getChatMsgDemo(); // #
        // # this.chatMsgs = this.getChatMsg(2, "evelyn_allen", 10); // #
        // # setTimeout(() => {
        // #     this.chatMsgs = this.getChatMsg(1, "evelyn_allen", 10); // #
        // #     this.changeDetector.markForCheck();
        // # }, 5000);
        // # setTimeout(() => {
        // #     this.chatMsgs = this.getChatMsg(0, "evelyn_allen", 10); // #
        // #     this.changeDetector.markForCheck();
        // # }, 10000);
        // =^
        // StreamState = [preparing, started, paused]
        this.isStreamActive = !!this.streamDto && StreamStateUtil.isActive(this.streamDto.state);
        console.log(`PgConceptView() this.isStreamActive: ${this.isStreamActive}`); // #

        const nickname: string = this.profileDto?.nickname || '';
        const streamId: number = (!!this.streamDto ? this.streamDto.id : -1);
        const access = this.profileTokensDto?.accessToken;

        if (!!nickname && !!streamId && this.isStreamActive) {
            this.chatSocketService.config({ nickname, room: streamId, access });

            this.chatSocketService.handlOnError = (err: string) => {
                console.error(`SocketErr:`, err);
                this.alertService.showError(err, 'pg-concept-view.error_socket');
                this.changeDetector.markForCheck();
            };
            this.chatSocketService.handlReceive = (val: string) => {
                console.log(`PgConceptView handlReceive(); changeDetector.markForCheck();`); // #
                this.handlReceiveChat(val);
                this.changeDetector.markForCheck();
            };
            // Connect to the server web socket chat.
            console.log(`PgConceptView.openSocket() socketSrv.connect()`); // #
            this.chatSocketService.connect(WS_CHAT_PATHNAME, WS_CHAT_HOST);
        }
    }

    ngOnInit(): void {
        this.updateParams(this.streamDto, this.profileDto);
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
        if (!!id && !!msgPut) {
            this.chatSocketService.sendData(EWSTypeUtil.getMsgPutEWS(msgPut, id));
        }
    }
    public doRemoveMessage(id: number | null): void {
        if (!!id) {
            this.chatSocketService.sendData(EWSTypeUtil.getMsgCutEWS('', id));
        }
    }
    public doQueryChatMsgs(info: { isSortDes: boolean, borderById: number }) {
        const streamId: number = (!!this.streamDto ? this.streamDto.id : -1);
        if (streamId > 0 && !!info && info.isSortDes != null && info.borderById != null) {
            console.log(`PgConceptView.doQueryPastInfo(${JSON.stringify(info)});`); // #
            this.getChatMessages(streamId, info.isSortDes, info.borderById, undefined);
        }
    }

    // ** Private API **

    private updateParams(streamDto: StreamDto | null, profileDto: ProfileDto | null): void {
        let isShowTimer = false;
        let isStreamOwner = false;
        if (!!streamDto) {
            isShowTimer = !!streamDto && StreamStateUtil.isActive(streamDto.state);
            const currentUserId: number = profileDto?.id || -1;
            isStreamOwner = (streamDto.userId === currentUserId);
        }
        this.isShowTimer = isShowTimer;
        this.isStreamOwner = isStreamOwner;
    }

    // Section: "panel stream admin"

    private toggleStreamState(isStreamOwner: boolean, streamId: number | null, streamState: StreamState | null): void {
        if (!isStreamOwner || !streamId || streamId < 0 || !streamState) {
            return;
        }
        this.isLoadStream = true;
        this.streamService.toggleStreamState(streamId, streamState)
            .then((response: StreamDto | HttpErrorResponse) => {
                this.streamDto = (response as StreamDto);
                this.updateParams(this.streamDto, this.profileDto);
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

    private handlReceiveChat = (val: string): void => {
        const eventWS = EventWS.parse(val);
        if (!eventWS) {
            return;
        }
        if (eventWS.et == EWSType.Err) {
            const err = eventWS.getStr('err') || '';
            console.error(`Socket-err:`, err);
            this.alertService.showError(err, 'pg-concept-view.error_socket');
        } else if (eventWS.et == EWSType.Echo) {
            const err = eventWS.getStr('echo') || '';
            console.log(`Socket-echo:`, err);
        } else if (eventWS.et == EWSType.Msg) {
            this.loadDataFromSocket(val);
        } else if (eventWS.et == EWSType.Block || eventWS.et == EWSType.Unblock) {
            const isBlock = eventWS.et == EWSType.Block;
            const user = isBlock ? eventWS.getStr('block') : eventWS.getStr('unblock');
            if (!!user && this.isStreamOwner) {
                this.blockedUsers = this.updateBlockedUsers(this.blockedUsers, isBlock, user);
            }
        }
    };
    private loadDataFromSocket(val: string): void {
        console.log(`PgConceptView.loadDataFromSocket(${val})`); // #
        const obj = JSON.parse(val);
        if (!!obj['id'] && !!obj['member'] && !!obj['date']) {
            const chatMsg = ChatMessageDtoUtil.create(obj);
            if (chatMsg.id > 0 && !!chatMsg.member && !!chatMsg.date) {
                this.chatMsgs = [chatMsg];
                console.log(`PgConceptView() 2 chatMsgs:`, this.chatMsgs); // #
            }
        }
    }
    private updateBlockedUsers(blockedUsers: string[], isBlock: boolean, blockUser: string): string[] {
        console.log(`PgConceptView.updateBlockedUsers(isBlock: ${isBlock}, blockUser: ${blockUser});`); // #
        const userSet: Set<string> = new Set(blockedUsers);
        if (isBlock) {
            userSet.add(blockUser);
        } else {
            userSet.delete(blockUser);
        }
        return Array.from(userSet);
    }
    private getChatMessages(streamId: number | null, isSortDes?: boolean, borderById?: number, limit?: number): void {
        console.log(`PgConceptView.getChatMessages(isSortDes: ${isSortDes}, borderById: ${borderById})...`); // #
        if (!streamId || streamId < 0) {
            return;
        }
        this.isLoadChatMsg = true;
        this.chatMessageService.getChatMessages(streamId, isSortDes, borderById, limit)
            .then((response: ChatMessageDto[] | HttpErrorResponse | undefined) => {
                console.log(`PgConceptView.getChatMessages() this.setChatMsgs(response)`); // #
                this.chatMsgs = (response as ChatMessageDto[]).concat([]);
                console.log(`PgConceptView() 3 chatMsgs:`, this.chatMsgs); // #
            })
            .catch((error: HttpErrorResponse) => {
                console.error(`ChatMessageError:`, error);
            })
            .finally(() => {
                this.isLoadChatMsg = false;
                this.changeDetector.markForCheck();
            });
    }
    /*private getChatMsgDemo(): ChatMessageDto[] {
        return [
            { "msg": "Demo msg Av1", "id": 1, "member": "emma_johnson", "date": "2025-04-28T09:10:30.727Z", "isEdt": false, "isRmv": false },
            { "msg": "Demo msg Bv2", "id": 2, "member": "evelyn_allen", "date": "2025-04-28T09:11:06.542Z", "isEdt": true, "isRmv": false },
            { "msg": "", "id": 3, "member": "liam_smith", "date": "2025-04-28T09:15:15.353Z", "isEdt": false, "isRmv": true },
            { "msg": "Demo msg Dv1", "id": 4, "member": "emma_johnson", "date": "2025-04-28T09:10:30.727Z", "isEdt": false, "isRmv": false },
            { "msg": "demo", "id": 5, "member": "evelyn_allen", "date": "2025-04-30T12:39:33.405Z", "isEdt": false, "isRmv": false },
            { "msg": "demo01", "id": 6, "member": "emma_johnson", "date": "2025-05-05T14:56:29.270Z", "isEdt": false, "isRmv": false },
            { "msg": "Demo02", "id": 7, "member": "evelyn_allen", "date": "2025-05-06T10:18:21.147Z", "isEdt": false, "isRmv": false },
            { "msg": "Demo03 v2", "id": 8, "member": "emma_johnson", "date": "2025-05-06T07:27:00.766Z", "isEdt": true, "isRmv": false },
            { "msg": "", "id": 9, "member": "evelyn_allen", "date": "2025-05-06T07:32:29.427Z", "isEdt": true, "isRmv": true },
            { "msg": "Demo02", "id": 10, "member": "evelyn_allen", "date": "2025-05-06T10:18:21.147Z", "isEdt": false, "isRmv": false },
        ];
    }*/

    /*private getChatMsg(mode: number, nickname: string, len: number): ChatMessageDto[] {
        const result: ChatMessageDto[] = [];
        const startId = 10 * (mode + 1);
        const currDate = new Date();
        currDate.setDate(currDate.getDate() - mode);
        currDate.setTime(currDate.getTime() - (2 * 60 * 60 * 1000)); // minus 2 hours
        const txt = "This function can be used to pass through a successful result while handling an error.";
        // const txt = "The Result pattern encapsulates the outcome of an operation in a Result object, which represents either a successful"
        //     + " result (containing the expected data) or a failure (with an error message).";
        // "Instead of throwing exceptions or returning null to indicate failure, you can use a Result to make each outcome clear and explicit."
        for (let idx = 0; idx < len; idx++) {
            let member = "Teodor_Nickols";
            const d1 = new Date(currDate.getTime());
            d1.setTime(d1.getTime() + (idx * 60 * 60 * 1000));
            // let d1 = new Date((idx < (len / 2) ? -100000000 : 0) + Date.now());
            let date = d1.toISOString();
            let id = startId - idx;
            let msg = "text_" + id + " " + txt;  // txt.slice(Math.random() * txt.length)
            if (idx % 3 == 0) {
                member = nickname;
            } else if (idx % 2 == 0) {
                member = "Snegana_Miller";
            }
            // const date1 = date.slice(20, 24) + '_' + date.slice(11, 19) + '_' + date.slice(0, 10);

            result.push({ id, date, member, msg, isEdt: false, isRmv: false });

        }
        return result;
    }*/

}
