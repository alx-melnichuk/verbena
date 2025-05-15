import { CommonModule, KeyValue } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, OnDestroy, OnInit, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';
import { StringDateTime } from 'src/app/common/string-date-time';

import { ConceptViewComponent } from 'src/app/lib-concept/concept-view/concept-view.component';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { ConfirmationData } from 'src/app/lib-dialog/confirmation/confirmation.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { ProfileDto, ProfileTokensDto } from 'src/app/lib-profile/profile-api.interface';
import { EWSTypeUtil } from 'src/app/lib-socket/socket-chat.interface';
import { SocketChatService } from 'src/app/lib-socket/socket-chat.service';
import { StreamDto, StreamState, StreamStateUtil } from 'src/app/lib-stream/stream-api.interface';
import { ChatMsg, ChatMsgUtil } from 'src/app/lib-stream/stream-chats.interface';
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
    providers: [SocketChatService],
})
export class PgConceptViewComponent implements OnInit, OnDestroy {

    public chatMaxRows: number = 4;
    public chatMinRows: number = 1;
    public isLoadStream = false;
    public isShowTimer: boolean = false;
    // An indication that the stream is in active status. ([preparing, started, paused]) 
    public isStreamActive: boolean = false;
    // An indication that this is the owner of the stream.
    public isStreamOwner = false;
    // The interval for displaying the timer before starting (in minutes).
    public showTimerBeforeStart: number | null | undefined;

    public profileDto: ProfileDto | null = null;
    public profileTokensDto: ProfileTokensDto | null = null;
    public streamDto: StreamDto | null = null;

    public chatName: string = '';
    public chatRoom: string = '';
    public chatAccess: string | null = null;
    public chatMsgs: ChatMsg[] = [];

    constructor(
        private changeDetector: ChangeDetectorRef,
        private route: ActivatedRoute,
        private translateService: TranslateService,
        private alertService: AlertService,
        private dialogService: DialogService,
        private streamService: StreamService,
        public socketChatService: SocketChatService,
    ) {
        // this.showTimerBeforeStart = 120; // minutes
        this.profileDto = this.route.snapshot.data['profileDto'];
        this.profileTokensDto = this.route.snapshot.data['profileTokensDto'];
        this.streamDto = this.route.snapshot.data['streamDto'];
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
        // this.chatMsgs = this.getChatMsgDemo(); // #
        this.chatMsgs = this.getChatMsg(2, "evelyn_allen", 10); // #
        // setTimeout(() => {
        //     this.chatMsgs = this.getChatMsg(1, "evelyn_allen", 10); // #
        //     this.changeDetector.markForCheck();
        // }, 5000);
        // setTimeout(() => {
        //     this.chatMsgs = this.getChatMsg(0, "evelyn_allen", 10); // #
        //     this.changeDetector.markForCheck();
        // }, 10000);
        // =^
        // StreamState = [preparing, started, paused]
        this.isStreamActive = !!this.streamDto && StreamStateUtil.isActive(this.streamDto.state);
        console.log(`PgConceptView() this.isStreamActive: ${this.isStreamActive}`); // #

        const nickname: string = this.profileDto?.nickname || '';
        const streamId: number = (!!this.streamDto ? this.streamDto.id : -1);
        const access = this.profileTokensDto?.accessToken;

        if (!!nickname && !!streamId && this.isStreamActive) {
            this.socketChatService.config({ nickname, room: streamId, access });

            this.socketChatService.handlOnError = (err: string) => {
                console.log(`PgConceptView handlOnError(); changeDetector.markForCheck();`); // #
                this.changeDetector.markForCheck();
            };
            this.socketChatService.handlReceive = (val: string) => {
                let s1 = ''; // #
                const obj = JSON.parse(val);
                if (!!obj['member'] && !!obj['msg']) {
                    const chatMsg: ChatMsg = ChatMsgUtil.create(JSON.parse(val));
                    s1 = ` chatMsg: ${JSON.stringify(chatMsg)}`; // #
                    this.chatMsgs = [chatMsg];
                }
                console.log(`PgConceptView handlReceive(); changeDetector.markForCheck();${s1}`); // #
                this.changeDetector.markForCheck();
            };
            // Connect to the server web socket chat.
            console.log(`PgConceptView.openSocket() socketSrv.connect()`); // #
            this.socketChatService.connect(WS_CHAT_PATHNAME, WS_CHAT_HOST);
        }
    }

    ngOnInit(): void {
        this.updateParams(this.streamDto, this.profileDto);
    }

    ngOnDestroy(): void {
        this.socketChatService.disconnect();
    }

    // ** Public API **

    // Section: "panel stream admin"

    public doChangeState(isStreamOwner: boolean, streamId: number | undefined, newState: StreamState | null): void {
        if (!!isStreamOwner && !!streamId && !!newState) {
            this.toggleStreamState(isStreamOwner, streamId, newState);
        }
    }

    // Section: "Chat"

    public doSendMessage(newMessage: string | null): void {
        const msgVal = (newMessage || '').trim();
        if (!!msgVal) {
            this.socketChatService.sendData(EWSTypeUtil.getMsgEWS(msgVal));
        }
    }
    public doEditMessage(keyValue: KeyValue<number, string> | null): void {
        const id = keyValue?.key;
        const msgPut = (keyValue?.value || '').trim();
        if (!!id && !!msgPut) {
            this.socketChatService.sendData(EWSTypeUtil.getMsgPutEWS(msgPut, id));
        }
    }
    public doRemoveMessage(id: number | null): void {
        if (!!id) {
            this.socketChatService.sendData(EWSTypeUtil.getMsgCutEWS('', id));
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

    // Stream for Owner

    private toggleStreamState(isStreamOwner: boolean, streamId: number | null, streamState: StreamState | null): void {
        if (!isStreamOwner || !streamId || !streamState) {
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
                    this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], title);
                }
            })
            .finally(() => {
                this.isLoadStream = false;
                this.changeDetector.markForCheck();
            });

    }

    private getChatMsgDemo(): ChatMsg[] {
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
    }

    private getChatMsg(mode: number, nickname: string, len: number): ChatMsg[] {
        const result: ChatMsg[] = [];
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
    }

}
