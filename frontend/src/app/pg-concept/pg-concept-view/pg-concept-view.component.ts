import { CommonModule, KeyValue } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, OnDestroy, OnInit, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';

import { ConceptViewComponent } from 'src/app/lib-concept/concept-view/concept-view.component';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { ConfirmationData } from 'src/app/lib-dialog/confirmation/confirmation.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';
import { SocketApiService } from 'src/app/lib-socket/socket-api.service';
import { ChatConfig, SocketChatService } from 'src/app/lib-socket/socket-chat.service';
import { StreamDto, StreamState, StreamStateUtil } from 'src/app/lib-stream/stream-api.interface';
import { RefreshChatMsgs } from 'src/app/lib-stream/stream-chats.interface';
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
    providers: [SocketApiService, SocketChatService],
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
    public streamDto: StreamDto | null = null;

    public chatName: string = '';
    public chatRoom: string = '';
    public chatAccess: string | null = null;

    private refreshChatMsgs: RefreshChatMsgs | null = null;

    constructor(
        private changeDetector: ChangeDetectorRef,
        private route: ActivatedRoute,
        private translateService: TranslateService,
        private alertService: AlertService,
        private dialogService: DialogService,
        private streamService: StreamService,
        // private socketApiService: SocketApiService,
        public socketChatService: SocketChatService,
    ) {
        // this.showTimerBeforeStart = 120; // minutes
        this.profileDto = this.route.snapshot.data['profileDto'];
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
        // =^
        // StreamState = [preparing, started, paused]
        this.isStreamActive = !!this.streamDto && StreamStateUtil.isActive(this.streamDto.state);
        console.log(`PgConceptView() this.isStreamActive: ${this.isStreamActive}`); // #

        const nickname: string = this.profileDto?.nickname || '';
        const streamId: string = (!!this.streamDto ? this.streamDto.id.toString() : '');

        if (!!nickname && !!streamId && this.isStreamActive) {
            this.socketChatService.config({ nickname, room: 'room_' + streamId });
            // // Connect to the server web socket chat.
            // this.socketChatService.connect(WS_CHAT_PATHNAME, WS_CHAT_HOST);
            console.log(`PgConceptView() socketChatService.connect2()`); // #
            this.socketChatService.connect2(WS_CHAT_PATHNAME, WS_CHAT_HOST)
                .then(() => {
                    console.log(`PgConceptView() socketChatService.connect2().then()`); // #
                })
                .catch(() => {
                    console.log(`PgConceptView() socketChatService.connect2().catch()`); // #

                })
                .finally(() => {
                    console.log(`PgConceptView() socketChatService.connect2().finally()`); // #
                    this.changeDetector.markForCheck();
                })
        }
    }

    ngOnInit(): void {
        console.log(`PgConceptView.OnInit()`); // #
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
        if (!msgVal) {
            return;
        }
        console.log(`doSendMessage(${msgVal})`); // #
    }
    public doEditMessage(keyValue: KeyValue<string, string> | null): void {
        const msgDate = (keyValue?.key || '').trim();
        const msgVal = (keyValue?.value || '').trim();
        if (!msgDate || !msgVal) {
            return;
        }
        console.log(`doEditMessage(${JSON.stringify(keyValue)})`); // #
    }
    public doRemoveMessage(messageDate: string | null): void {
        const msgDate = (messageDate || '').trim();
        if (!msgDate) {
            return;
        }
        console.log(`doRemoveMessage(${messageDate})`); // #
    }
    public doSetRefreshChatMsgs(refreshChatMsgs: RefreshChatMsgs | null): void {
        if (!!refreshChatMsgs) {
            console.log(`doSetRefreshChatMsgs()`); // #
            this.refreshChatMsgs = refreshChatMsgs;
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

}
